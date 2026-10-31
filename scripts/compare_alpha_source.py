#!/usr/bin/env python3
"""Compare the native alpha workflow with the retained, qualified source capture."""

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import platform
import subprocess
import sys

WITNESS_SHA256 = 'b6eb9045c2dceba4754821f90c8c553a3da221f04405364bbd0639fd4fd146a1'
MANIFEST_SHA256 = '34b96bfccadb55f6a2ff57e443c8089a1b3636171bea7be6b89c4d86ef727439'
BOUNDS = [1e-10, 1e-10]


def sha256(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def load(path):
    def invalid(value):
        raise ValueError(f'invalid JSON nonfinite token: {value}')
    return json.loads(path.read_text(), parse_constant=invalid)


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')


def compare(args, report):
    witness_path = args.witness / 'witness.json'
    assert sha256(witness_path) == WITNESS_SHA256, 'source witness identity changed'
    assert sha256(args.witness / 'sha256-manifest.tsv') == MANIFEST_SHA256, 'source manifest identity changed'
    witness = load(witness_path)
    assert witness['status'] == 'complete_source_capture'
    assert witness['bounds']['alpha'] == witness['bounds']['phylogenetic'] == BOUNDS
    assert witness['runtime']['packages']['numpy']['version'] == '2.5.4'
    assert witness['runtime']['packages']['scipy']['version'] == '1.18.1'
    report.update(source_witness_sha256=WITNESS_SHA256, source_manifest_sha256=MANIFEST_SHA256,
                  source_pins=witness['source_pins'], source_packages=witness['runtime']['packages'],
                  bounds={'absolute': BOUNDS[0], 'relative': BOUNDS[1]},
                  binary={'path': str(args.binary), 'sha256': sha256(args.binary)},
                  source_inputs={})

    def source(relative):
        path = args.witness / relative
        assert sha256(path) == witness['files'][relative], f'changed source file: {relative}'
        report['source_inputs'][relative] = witness['files'][relative]
        return load(path)

    cases = [('original', 'cases/original'), ('rarefied', 'cases/rarefied/diversity')]
    cases += [(name, 'source-boundaries/' + name) for name in
              ('zero_samples', 'no_features', 'root_length', 'zero_lengths', 'extra_tips', 'unary', 'polytomy')]
    for name, relative in cases:
        request = source(relative + '/request.json')
        folder = args.output / name
        folder.mkdir()
        counts, tree = folder / 'counts.tsv', folder / 'tree.nwk'
        if name == 'rarefied' and args.rarefied_counts is not None:
            counts = args.rarefied_counts
            with counts.open(newline='') as stream:
                rows = list(csv.reader(stream, delimiter='\t'))
            assert rows[0][0] in ('feature', '#OTU ID')
            assert rows[0][1:] == request['sample_ids']
            assert [row[0] for row in rows[1:]] == request['feature_ids']
            assert [[int(value) for value in row[1:]] for row in rows[1:]] == list(map(list, zip(*request['counts'])))
            report['native_rarefied_counts'] = {'path': str(counts), 'sha256': sha256(counts)}
        else:
            with counts.open('w', newline='') as stream:
                writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
                writer.writerow(['#OTU ID', *request['sample_ids']])
                for feature, values in zip(request['feature_ids'], zip(*request['counts'])):
                    assert all(type(value) is int and value >= 0 for value in values)
                    writer.writerow([feature, *values])
        tree.write_text(request['tree'])
        expected = {}
        for metric in witness['alpha_metrics'] + ['faith_pd'] + ['phydiv-' + profile[0] for profile in witness['phydiv_profiles']]:
            path = relative + '/alpha/' + metric + '/observation.json'
            if path not in witness['files']:
                continue
            observation = source(path)
            if observation['status'] == 'returned':
                expected[metric] = observation['result']
            else:
                assert name == 'polytomy' and metric == 'faith_pd', (name, metric, observation)
        for profile, rooted, weight in witness['phydiv_profiles']:
            phydiv_name = 'phydiv-' + profile
            if phydiv_name not in expected:
                continue
            metrics = [metric for metric in witness['alpha_metrics'] + ['faith_pd'] if metric in expected] if profile == 'auto' else []
            metrics += ['phydiv']
            references = [expected[metric] if metric != 'phydiv' else expected[phydiv_name] for metric in metrics]
            native_reports = []
            for workers in (1, 4):
                destination = folder / f'{profile}-{workers}.json'
                command = [str(args.binary), 'alpha', '--counts', str(counts), '--tree', str(tree),
                           '--output', str(destination), '--format', 'json', '--json', '--threads', str(workers),
                           '--metrics', ','.join(metrics), '--pd-rooted', 'auto' if rooted is None else 'rooted' if rooted else 'unrooted',
                           '--pd-weight', str(int(weight) if isinstance(weight, bool) else weight)]
                completed = subprocess.run(command, capture_output=True, text=True)
                prefix = folder / f'{profile}-{workers}'
                prefix.with_suffix('.stdout').write_text(completed.stdout)
                prefix.with_suffix('.stderr').write_text(completed.stderr)
                entry = dict(case=name, profile=profile, workers=workers, command=command, exit_status=completed.returncode)
                report['runs'].append(entry)
                assert completed.returncode == 0, completed.stderr
                envelope = json.loads(completed.stdout)
                assert envelope['status'] == 'ok' and envelope['result']['workers'] == workers
                actual = load(destination)
                assert actual['schema_version'] == 1
                assert actual['feature_ids'] == request['feature_ids']
                assert actual['sample_ids'] == request['sample_ids']
                assert actual['metrics'] == metrics
                assert len(actual['values']) == len(request['sample_ids'])
                assert actual['pd']['root_mode'] == ('auto' if rooted is None else 'rooted' if rooted else 'unrooted')
                assert actual['pd']['weight'] == weight
                assert all(len(row) == len(metrics) for row in actual['values'])
                for column, reference in enumerate(references):
                    assert reference['kind'] == 'series' and reference['ids'] == request['sample_ids']
                    for row, expected_value in enumerate(reference['values']):
                        value = actual['values'][row][column]
                        if isinstance(expected_value, dict):
                            assert value['value'] is None and value['nonfinite'] == expected_value['nonfinite']
                            assert isinstance(value['reason'], str) and value['reason']
                            report['unavailable_values'] += 1
                        else:
                            number = value['value']
                            assert number is not None and math.isfinite(number)
                            difference = abs(number - expected_value)
                            assert difference <= BOUNDS[0] + BOUNDS[1] * abs(expected_value), (name, profile, row, metrics[column], number, expected_value)
                            report['maximum_absolute_error'] = max(report['maximum_absolute_error'], difference)
                        report['compared_values'] += 1
                entry.update(status='pass', result_sha256=sha256(destination), series=len(metrics))
                native_reports.append(actual)
            assert native_reports[0] == native_reports[1], (name, profile, 'worker-dependent result')
    assert sha256(args.binary) == report['binary']['sha256'], 'native executable changed during comparison'
    if args.rarefied_counts is not None:
        assert sha256(args.rarefied_counts) == report['native_rarefied_counts']['sha256'], 'native rarefied table changed during comparison'


def main():
    if not __debug__:
        raise RuntimeError('comparison checks require Python without -O or PYTHONOPTIMIZE')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--witness', type=Path, required=True)
    parser.add_argument('--rarefied-counts', type=Path,
                        help='native-rarefied primary table, checked against the retained source input')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.binary = args.binary.resolve(strict=True)
    args.witness = args.witness.resolve(strict=True)
    if args.rarefied_counts is not None:
        args.rarefied_counts = args.rarefied_counts.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    report = dict(status='failed', scope='retained source alpha comparison only; not a source replay, benchmark or release gate',
                  python=sys.version, machine=platform.platform(), runs=[], compared_values=0,
                  unavailable_values=0, maximum_absolute_error=0.0)
    exit_status = 0
    try:
        compare(args, report)
        report['status'] = 'pass'
    except Exception as error:
        report['error'] = f'{type(error).__name__}: {error}'
        exit_status = 1
    report['files'] = {str(path.relative_to(args.output)): sha256(path)
                       for path in sorted(args.output.rglob('*')) if path.is_file()}
    save(args.output / 'comparison.json', report)
    print(json.dumps({key: report[key] for key in ('status', 'compared_values', 'unavailable_values', 'maximum_absolute_error')}, allow_nan=False))
    return exit_status


if __name__ == '__main__':
    raise SystemExit(main())
