#!/usr/bin/env python3
"""Qualify native beta files against original40634594900 data, without source replay."""

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
METRICS = ('braycurtis', 'jaccard', 'euclidean', 'canberra', 'cityblock',
           'unweighted_unifrac', 'weighted_unifrac', 'weighted_normalized_unifrac')
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
    assert sha256(args.witness / 'witness.json') == WITNESS_SHA256
    assert sha256(args.witness / 'sha256-manifest.tsv') == MANIFEST_SHA256
    witness = load(args.witness / 'witness.json')
    assert witness['status'] == 'complete_source_capture'
    assert witness['bounds']['beta'] == BOUNDS
    assert witness['runtime']['packages']['scipy']['version'] == '1.18.1'
    assert witness['runtime']['packages']['numpy']['version'] == '2.5.4'
    frozen = Path(__file__).resolve().parents[1] / 'tests/data/beta'
    hashes = load(frozen / 'source-files.json')
    assert hashes['witness_sha256'] == WITNESS_SHA256
    assert hashes['manifest_sha256'] == MANIFEST_SHA256
    for relative, digest in hashes['files'].items():
        assert digest == witness['files'][relative] == sha256(frozen / relative), relative
    report.update(source_witness_sha256=WITNESS_SHA256, source_manifest_sha256=MANIFEST_SHA256,
                  source_pins=witness['source_pins'], source_packages=witness['runtime']['packages'],
                  binary={'path': str(args.binary), 'sha256': sha256(args.binary)}, source_inputs={},
                  bounds={'absolute': BOUNDS[0], 'relative': BOUNDS[1]},
                  frozen_source_files=hashes['files'])

    def source(relative):
        path = args.witness / relative
        assert sha256(path) == witness['files'][relative], relative
        report['source_inputs'][relative] = witness['files'][relative]
        return load(path)

    def run(command, prefix, **metadata):
        completed = subprocess.run(command, capture_output=True, text=True)
        Path(str(prefix) + '.stdout').write_text(completed.stdout)
        Path(str(prefix) + '.stderr').write_text(completed.stderr)
        entry = dict(command=command, exit_status=completed.returncode, **metadata)
        report['runs'].append(entry)
        return completed, entry

    cases = [('original', 'cases/original'), ('rarefied', 'cases/rarefied/diversity')]
    cases += [(name, 'source-boundaries/' + name) for name in
              ('zero_samples', 'no_features', 'root_length', 'zero_lengths', 'extra_tips', 'unary', 'polytomy')]
    original_counts = None
    for name, relative in cases:
        request = source(relative + '/request.json')
        folder = args.output / name
        folder.mkdir()
        counts, tree = folder / 'counts.tsv', folder / 'tree.nwk'
        if name == 'rarefied':
            previous = None
            for workers in (1, 4):
                command = [str(args.binary), '--json', '--threads', str(workers), 'rarefy',
                           '--counts', str(original_counts), '--output', str(counts), '--depth', '8', '--seed', '42']
                completed, entry = run(command, folder / f'rarefy-{workers}', operation='rarefy', workers=workers)
                assert completed.returncode == 0, completed.stderr
                assert load_envelope(completed.stdout)['result']['workers'] == workers
                current = counts.read_bytes()
                expected = (frozen.parent / 'rarefaction/primary-expected.tsv').read_bytes()
                assert current == expected.replace(b'#OTU ID\t', b'feature\t', 1)
                if previous is not None:
                    assert current == previous
                previous = current
                entry.update(status='pass', result_sha256=sha256(counts))
            with counts.open(newline='') as stream:
                rows = list(csv.reader(stream, delimiter='\t'))
            assert rows[0][1:] == request['sample_ids']
            assert [row[0] for row in rows[1:]] == request['feature_ids']
            assert [[int(v) for v in row[1:]] for row in rows[1:]] == list(map(list, zip(*request['counts'])))
            report['native_rarefied_counts'] = {'path': str(counts), 'sha256': sha256(counts)}
        else:
            with counts.open('w', newline='') as stream:
                writer = csv.writer(stream, delimiter='\t', lineterminator='\n')
                writer.writerow(['feature', *request['sample_ids']])
                for feature, values in zip(request['feature_ids'], zip(*request['counts'])):
                    assert all(type(value) is int and value >= 0 for value in values)
                    writer.writerow([feature, *values])
        if name == 'original':
            original_counts = counts
        tree.write_text(request['tree'])
        for metric in METRICS:
            observation_path = relative + '/beta/' + metric + '/observation.json'
            if observation_path not in witness['files']:
                assert name in ('root_length', 'zero_lengths', 'extra_tips', 'unary', 'polytomy') and 'unifrac' not in metric
                continue  # The original bounded capture requested only UniFrac in these groups.
            observation = source(observation_path)
            first_undefined = None
            if observation['status'] == 'returned':
                reference = observation['result']
                n = len(request['sample_ids'])
                assert reference['kind'] == 'distance-matrix'
                assert reference['ids'] == request['sample_ids'] and reference['shape'] == [n, n]
                assert len(reference['values']) == n and all(len(row) == n for row in reference['values'])
                for i, row in enumerate(reference['values']):
                    for j, value in enumerate(row):
                        if isinstance(value, dict):
                            assert value == {'nonfinite': 'nan'} and i != j
                            report['source_nonfinite_observations'] += 1
                            if i < j and first_undefined is None:
                                first_undefined = (i, j)
                if first_undefined is not None:
                    assert (name, metric) in {('zero_samples', 'braycurtis'), ('zero_lengths', 'weighted_normalized_unifrac')}
            else:
                assert name == 'polytomy' and 'unifrac' in metric
            error_expected = first_undefined is not None or observation['status'] != 'returned'
            previous = {}
            for workers in (1, 4):
                json_matrix = None
                for format in ('json', 'tsv'):
                    destination = folder / f'{metric}-{workers}.{format}'
                    if error_expected:
                        destination.write_bytes(b'keep\n')
                    command = [str(args.binary), '--json', '--threads', str(workers), 'beta',
                               '--counts', str(counts), '--output', str(destination), '--metric', metric, '--format', format]
                    if 'unifrac' in metric:
                        command += ['--tree', str(tree)]
                    completed, entry = run(command, folder / f'{metric}-{workers}-{format}', case=name, metric=metric,
                                           workers=workers, format=format, source_status=observation['status'],
                                           native_expected_status='error' if error_expected else 'finite_matrix',
                                           source_observation_sha256=witness['files'][observation_path])
                    if error_expected:
                        assert completed.returncode != 0 and not completed.stdout
                        envelope = json.loads(completed.stderr)
                        assert envelope['status'] == 'error'
                        if first_undefined is not None:
                            i, j = first_undefined
                            assert metric in completed.stderr
                            assert request['sample_ids'][i] in completed.stderr and request['sample_ids'][j] in completed.stderr
                        assert destination.read_bytes() == b'keep\n'
                        if format in previous:
                            assert completed.stderr == previous[format]
                        previous[format] = completed.stderr
                        entry.update(status='pass', native_observed_status='error', preserved_destination=True)
                        continue
                    assert completed.returncode == 0 and not completed.stderr, completed.stderr
                    envelope = load_envelope(completed.stdout)
                    assert envelope['result']['workers'] == workers
                    assert envelope['result']['beta']['metric'] == metric
                    if format == 'json':
                        actual = load(destination)
                        assert actual['schema_version'] == 1 and actual['metric'] == metric
                        assert actual['sample_ids'] == request['sample_ids']
                        assert actual['source_profile'] == 'skbio-0.7.3/scipy-1.18.1'
                        assert actual['tree_profile'] == ('source-root-degree<=2/stored-root-length/strict-nonnegative' if 'unifrac' in metric else None)
                        matrix = actual['values']
                        json_matrix = matrix
                    else:
                        with destination.open(newline='') as stream:
                            rows = list(csv.reader(stream, delimiter='\t'))
                        assert rows[0] == ['', *request['sample_ids']]
                        assert [row[0] for row in rows[1:]] == request['sample_ids']
                        matrix = [[float(v) for v in row[1:]] for row in rows[1:]]
                        assert matrix == json_matrix, (name, metric, 'TSV/JSON round-trip mismatch')
                    assert len(matrix) == n and all(len(row) == n for row in matrix)
                    for i, row in enumerate(matrix):
                        for j, value in enumerate(row):
                            expected = reference['values'][i][j]
                            assert math.isfinite(value) and value >= 0 and value == matrix[j][i]
                            assert i != j or value == 0
                            difference = abs(value - expected)
                            assert difference <= BOUNDS[0] + BOUNDS[1] * abs(expected), (name, metric, i, j, value, expected)
                            report['maximum_absolute_error'] = max(report['maximum_absolute_error'], difference)
                            report['compared_values'] += 1
                    current = destination.read_bytes()
                    if format in previous:
                        assert current == previous[format], (name, metric, format, 'worker-dependent bytes')
                    previous[format] = current
                    entry.update(status='pass', result_sha256=sha256(destination))
    assert sha256(args.binary) == report['binary']['sha256'], 'binary changed during qualification'


def load_envelope(text):
    result = json.loads(text)
    assert result['status'] == 'ok'
    return result


def main():
    if not __debug__:
        raise RuntimeError('comparison requires Python without optimization')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--witness', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.binary = args.binary.resolve(strict=True)
    args.witness = args.witness.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    report = dict(status='failed', scope='retained beta data qualification; not source replay, performance or release',
                  python=sys.version, machine=platform.platform(), runs=[], compared_values=0,
                  source_nonfinite_observations=0, maximum_absolute_error=0.0)
    status = 0
    try:
        compare(args, report)
        report['status'] = 'pass'
    except Exception as error:
        report['error'] = f'{type(error).__name__}: {error}'
        status = 1
    report['files'] = {str(path.relative_to(args.output)): sha256(path)
                       for path in sorted(args.output.rglob('*')) if path.is_file()}
    save(args.output / 'comparison.json', report)
    print(json.dumps({key: report[key] for key in ('status', 'compared_values', 'source_nonfinite_observations', 'maximum_absolute_error')}))
    return status


if __name__ == '__main__':
    raise SystemExit(main())
