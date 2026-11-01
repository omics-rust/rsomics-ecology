"""Package, install and archive the count-table diversity CLI."""

import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile

PRODUCT = 'rsomics-ecology'
ROOT = Path(__file__).resolve().parents[1]
ALPHA = ('observed_features', 'shannon', 'simpson', 'dominance', 'inv_simpson',
         'simpson_e', 'pielou_e', 'chao1', 'goods_coverage', 'faith_pd', 'phydiv')
BETA = ('braycurtis', 'jaccard', 'euclidean', 'canberra', 'cityblock',
        'unweighted_unifrac', 'weighted_unifrac', 'weighted_normalized_unifrac')


def identity(path):
    body = Path(path).read_bytes()
    return dict(bytes=len(body), sha256=hashlib.sha256(body).hexdigest())


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n')


def require(ok, message):
    if not ok:
        raise ValueError(message)


def package_contract(package):
    parent = Path(package['manifest_path']).parent
    result = {k: v for k, v in package.items() if k not in ('id', 'manifest_path', 'targets')}
    result['targets'] = [{**t, 'src_path': str(Path(t['src_path']).relative_to(parent))} for t in package['targets']]
    return result


def extract_package(source, destination, expected, head):
    destination = Path(destination)
    destination.mkdir(parents=True)
    with tarfile.open(source, 'r:gz') as archive:
        members = archive.getmembers()
        require(len(members) == len({m.name for m in members}), 'duplicate source package member')
        bodies = {}
        for member in members:
            path = PurePosixPath(member.name)
            require(not path.is_absolute() and '..' not in path.parts and path.parts[0] == destination.name,
                    'unsafe source package path')
            require(member.isfile() or member.isdir(), 'unsupported source package member')
            if member.isfile():
                name = '/'.join(path.parts[1:])
                mode = expected[name]['mode'] if name in expected else 0o644
                require(name and member.mode == mode, 'source package file mode differs: ' + name)
                bodies[name] = archive.extractfile(member).read()
        require(set(bodies) == set(expected) | {'Cargo.toml.orig', '.cargo_vcs_info.json'},
                'complete source package membership differs')
        require(bodies['Cargo.toml.orig'] == expected['Cargo.toml']['body'], 'original Cargo manifest differs')
        vcs = json.loads(bodies['.cargo_vcs_info.json'])
        require(vcs['git']['sha1'] == head and not vcs['git'].get('dirty', False) and vcs['path_in_vcs'] == '',
                'source package is not the exact clean CI head')
        for name, row in expected.items():
            if name != 'Cargo.toml':
                require(bodies[name] == row['body'], 'source package body differs: ' + name)
        for name, body in bodies.items():
            path = destination / name
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open('xb') as stream:
                stream.write(body)
            path.chmod(expected[name]['mode'] if name in expected else 0o644)
    return dict(**identity(source), members=len(bodies), original_members=len(expected),
                original_bodies_modes_match=True, source_commit=head, clean_vcs=True)


def source():
    temporary = Path(os.environ['RUNNER_TEMP'])
    delivery = temporary / 'delivery'
    metadata = json.loads((delivery / 'package-metadata.json').read_bytes())
    package, = [p for p in metadata['packages'] if p['name'] == PRODUCT]
    prefix = PRODUCT + '-' + package['version']
    source = Path(metadata['target_directory']) / 'package' / (prefix + '.crate')
    shutil.copy2(source, delivery / source.name)
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    require(head == os.environ['GITHUB_SHA'], 'source head differs from actual CI head')
    require(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT) == b'', 'source worktree is dirty')
    expected = {}
    for row in subprocess.check_output(['git', 'ls-tree', '-rz', head], cwd=ROOT).rstrip(b'\0').split(b'\0'):
        header, name = row.split(b'\t')
        mode, kind, blob = header.decode().split()
        name = name.decode()
        require(kind == 'blob' and mode in ('100644', '100755'), 'unexpected source Git type/mode: ' + name)
        body = (ROOT / name).read_bytes()
        require(hashlib.sha1(b'blob ' + str(len(body)).encode() + b'\0' + body).hexdigest() == blob,
                'source differs from exact Git body: ' + name)
        expected[name] = dict(mode=int(mode, 8) & 0o777, body=body)
    destination = temporary / 'packaged-source' / prefix
    verification = extract_package(source, destination, expected, head)
    unpacked = json.loads(subprocess.check_output(['cargo', 'metadata', '--manifest-path', str(destination / 'Cargo.toml'),
                                                 '--locked', '--offline', '--no-deps', '--format-version', '1']))
    installed, = [p for p in unpacked['packages'] if p['name'] == PRODUCT]
    require(package_contract(installed) == package_contract(package), 'normalized source package metadata differs')
    verification['normalized_metadata_match'] = True
    save(delivery / 'package-verification.json', verification)
    metadata_body = subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT)
    tree_body = subprocess.check_output(['cargo', 'tree', '--locked', '--offline', '--target', os.environ['EXPECTED_HOST'],
                                        '--edges', 'normal,build', '--prefix', 'none', '--format', '{p}'], cwd=ROOT)
    (delivery / 'native-dependencies.metadata.json').write_bytes(metadata_body)
    (delivery / 'native-dependencies.tree.txt').write_bytes(tree_body)
    by_name = {(p['name'], p['version']): p for p in json.loads(metadata_body)['packages']}
    enabled = set()
    for line in tree_body.decode().splitlines():
        name, version = re.match(r'([^ ]+) v([^ ]+)', line).groups()
        if name != PRODUCT:
            enabled.add(by_name[name, version]['id'])
    inventory = json.loads((ROOT / 'licenses/provenance.json').read_bytes())
    require(enabled == set(inventory['target_graphs'][os.environ['EXPECTED_HOST']]), 'enabled target dependency inventory differs')
    require(identity(ROOT / 'Cargo.lock')['sha256'] == inventory['lock_sha256'], 'notice inventory lock differs')
    for row in inventory['copied_texts'].values():
        require(identity(ROOT / row['destination']) == {k: row[k] for k in ('bytes', 'sha256')}, 'bundled notice body differs')
    save(delivery / 'native-dependencies.json', dict(target=os.environ['EXPECTED_HOST'], enabled_packages=sorted(enabled),
         inventory_sha256=identity(ROOT / 'licenses/provenance.json')['sha256'],
         metadata_sha256=hashlib.sha256(metadata_body).hexdigest(), tree_sha256=hashlib.sha256(tree_body).hexdigest(),
         enabled_normal_build_graph_verified=True, linked_binary_inventory=False))
    with Path(os.environ['GITHUB_ENV']).open('a') as stream:
        stream.write('PACKAGE_DIRECTORY=' + str(destination) + '\n')
        stream.write('PACKAGE_VERSION=' + package['version'] + '\n')
        stream.write('SOURCE_PACKAGE=' + str(delivery / source.name) + '\n')


def near(actual, expected):
    require(type(actual) in (int, float) and math.isfinite(actual) and
            abs(actual - expected) <= 1e-10 + 1e-10 * abs(expected), 'installed diversity result differs')


def smoke(binary):
    binary = Path(binary).resolve(strict=True)
    directory = Path(os.environ['RUNNER_TEMP']) / 'delivery/lifecycle'
    directory.mkdir(parents=True)
    shutil.copy2(binary, directory / 'installed-binary')
    records = []

    def run(arguments, *, succeeds=True):
        command = [str(binary), *map(str, arguments)]
        index = len(records)
        row = dict(argv=command)
        records.append(row)
        save(directory / 'commands.json', records)
        with (directory / f'{index}.stdout').open('xb') as out, (directory / f'{index}.stderr').open('xb') as err:
            result = subprocess.run(command, cwd=directory, stdout=out, stderr=err,
                                    env=dict(os.environ, NO_COLOR='1'), timeout=120)
        row['returncode'] = result.returncode
        save(directory / 'commands.json', records)
        require((result.returncode == 0) == succeeds, 'installed command exit differs: ' + str(command))
        return directory / f'{index}.stdout', directory / f'{index}.stderr'

    def envelope(arguments):
        out, err = run(['--json', '--threads', 4, *arguments])
        require(err.stat().st_size == 0, 'installed successful command wrote diagnostics')
        value = json.loads(out.read_bytes())
        require(value['tool'] == PRODUCT and value['status'] == 'ok' and value['result']['workers'] == 4,
                'installed command envelope differs')
        return value['result']

    out, _ = run(['--version'])
    require(out.read_text().strip() == PRODUCT + ' ' + os.environ['PACKAGE_VERSION'], 'installed version differs')
    out, _ = run(['--help'])
    commands = out.read_text().split('Commands:\n', 1)[1].split('\n\n', 1)[0]
    require([line.split()[0] for line in commands.splitlines() if line.split()[0] != 'help'] ==
            ['alpha', 'beta', 'rarefy'], 'installed stable operation scope differs')
    fixture = ROOT / 'tests/data/rarefaction'
    captures = ROOT / 'tests/data/beta/cases/rarefied/diversity'
    inputs = directory / 'counts.tsv'
    tree = directory / 'tree.nwk'
    rarefied = directory / 'rarefied.tsv'
    shutil.copy2(fixture / 'primary-in.tsv', inputs)
    shutil.copy2(fixture / 'primary-tree.nwk', tree)
    report = envelope(['rarefy', '--counts', inputs, '--output', rarefied, '--depth', 8, '--seed', 42])
    require(report['features'] == 12 and report['samples'] == 6 and report['rarefaction'] ==
            dict(depth=8, seed='42', profile='skbio-0.7.3/biom-2.1.17/numpy-2.5.4-pcg64-choice-shuffle-false'),
            'installed rarefaction profile or dimensions differ')
    require(rarefied.read_bytes() == (fixture / 'primary-expected.tsv').read_bytes().replace(b'#OTU ID\t', b'feature\t', 1),
            'installed seeded rarefaction differs from retained source')
    request = json.loads((captures / 'request.json').read_bytes())
    alpha_path = directory / 'alpha.json'
    envelope(['alpha', '--counts', rarefied, '--tree', tree, '--metrics', ','.join(ALPHA),
              '--format', 'json', '--output', alpha_path])
    alpha = json.loads(alpha_path.read_bytes())
    require(alpha['schema_version'] == 1 and alpha['sample_ids'] == request['sample_ids'] and
            alpha['feature_ids'] == request['feature_ids'] and alpha['metrics'] == list(ALPHA),
            'installed alpha schema, identities or metric scope differ')
    require(len(alpha['values']) == 6 and all(len(row) == 11 for row in alpha['values']), 'installed alpha shape differs')
    for column, metric in enumerate(ALPHA):
        label = 'phydiv-auto' if metric == 'phydiv' else metric
        expected = json.loads((captures / 'alpha' / label / 'observation.json').read_bytes())
        require(expected['status'] == 'returned' and expected['result']['ids'] == request['sample_ids'],
                'retained alpha source identity differs')
        for row, value in enumerate(expected['result']['values']):
            near(alpha['values'][row][column]['value'], value)
    matrices = {}
    for metric in BETA:
        path = directory / (metric + '.json')
        arguments = ['beta', '--counts', rarefied, '--metric', metric, '--format', 'json', '--output', path]
        if 'unifrac' in metric:
            arguments += ['--tree', tree]
        result = envelope(arguments)
        require(result['beta']['metric'] == metric, 'installed beta envelope metric differs')
        actual = json.loads(path.read_bytes())
        expected = json.loads((captures / 'beta' / metric / 'observation.json').read_bytes())
        require(expected['status'] == 'returned' and expected['result']['ids'] == request['sample_ids'],
                'retained beta source identity differs')
        require(actual['schema_version'] == 1 and actual['sample_ids'] == request['sample_ids'] and
                actual['metric'] == metric and actual['source_profile'] == 'skbio-0.7.3/scipy-1.18.1' and
                actual['tree_profile'] == ('source-root-degree<=2/stored-root-length/strict-nonnegative'
                                          if 'unifrac' in metric else None), 'installed beta schema or profiles differ')
        values = actual['values']
        require(len(values) == 6 and all(len(row) == 6 for row in values), 'installed beta shape differs')
        for i, row in enumerate(values):
            for j, value in enumerate(row):
                near(value, expected['result']['values'][i][j])
                require(value >= 0 and value == values[j][i] and (i != j or value == 0),
                        'installed beta matrix is not finite, symmetric and hollow')
        matrices[metric] = values
    text = directory / 'braycurtis.tsv'
    envelope(['beta', '--counts', rarefied, '--metric', 'braycurtis', '--format', 'tsv', '--output', text])
    with text.open(newline='') as stream:
        rows = list(csv.reader(stream, delimiter='\t'))
    require(rows[0] == ['', *request['sample_ids']] and [row[0] for row in rows[1:]] == request['sample_ids'] and
            [[float(value) for value in row[1:]] for row in rows[1:]] == matrices['braycurtis'],
            'installed labelled TSV/JSON round-trip differs')
    zero, protected = directory / 'zero.tsv', directory / 'protected.json'
    zero.write_text('feature\tempty-a\tempty-b\nU1\t0\t0\n')
    protected.write_bytes(b'previous result\n')
    out, err = run(['--json', '--threads', 4, 'beta', '--counts', zero, '--metric', 'braycurtis',
                    '--format', 'json', '--output', protected], succeeds=False)
    diagnostic = err.read_text()
    require(out.stat().st_size == 0 and json.loads(diagnostic)['status'] == 'error' and
            all(name in diagnostic for name in ('braycurtis', 'empty-a', 'empty-b')) and
            protected.read_bytes() == b'previous result\n', 'installed undefined-pair failure lost atomicity or identities')
    save(directory / 'VERIFIED.json', dict(installed_binary=identity(binary), calls=records,
         stable_operations=['alpha', 'beta', 'rarefy'], beta_metrics=list(BETA), all_alpha_metrics_checked=11,
         features=12, samples=6, depth=8, seed='42', alpha_schema=1, beta_schema=1,
         retained_expected_source=True, absolute_band=1e-10, relative_band=1e-10,
         labelled_tsv_json_roundtrip=True, failure_preserves_prior_output=True,
         files={p.name: identity(p) for p in directory.iterdir() if p.is_file()},
         scope='installed count-table diversity lifecycle only; not new live source execution or performance'))


def bundle(binary):
    temporary = Path(os.environ['RUNNER_TEMP'])
    delivery = temporary / 'delivery'
    binary = Path(binary).resolve(strict=True)
    stem = PRODUCT + '-' + os.environ['PACKAGE_VERSION'] + '-' + os.environ['NATIVE_TARGET']
    folder = delivery / stem
    folder.mkdir()
    shutil.copy2(binary, folder / PRODUCT)
    for name in ('README.md', 'LICENSE-MIT', 'LICENSE-APACHE-2.0', 'THIRD_PARTY_LICENSES.md', 'THIRD_PARTY_NOTICES.md'):
        shutil.copy2(ROOT / name, folder / name)
    shutil.copytree(ROOT / 'licenses', folder / 'licenses')
    provenance = dict(product=PRODUCT, version=os.environ['PACKAGE_VERSION'], target=os.environ['NATIVE_TARGET'],
         head_sha=os.environ['GITHUB_SHA'], run_id=int(os.environ['GITHUB_RUN_ID']), run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']),
         native_host=os.environ['EXPECTED_HOST'], rustc=subprocess.check_output(['rustc', '-vV'], text=True),
         binary=identity(binary), lock_sha256=identity(ROOT / 'Cargo.lock')['sha256'],
         source_package=json.loads((delivery / 'package-verification.json').read_bytes()),
         native_dependencies=json.loads((delivery / 'native-dependencies.json').read_bytes()),
         accepted_beta_implementation='111adf5e8a3657a99c50a9832ca5ed9c53fd98d7',
         historical_beta_binary_sha256='d7dc1832fab7857aba17179b4eb632438094b7f64c7ac7bf562bd64778017bd3',
         historical_alpha_rarefaction_implementation='e1b03cb68876be2dd8bb9fbc520b5194be055d9e',
         historical_alpha_rarefaction_binary_sha256='58cd2ec1e4072ee4e76d2a244c2547022524025faf6b406af8cd65ef08d45a0a',
         this_binary_remeasured=False, registry_publication=False, signed_release=False)
    save(folder / 'BUILD.json', provenance)
    for path in [folder, *folder.rglob('*')]:
        path.chmod(0o755 if path.is_dir() or path.name == PRODUCT else 0o644)
    archive = delivery / (stem + '.tar.gz')
    with tarfile.open(archive, 'w:gz') as output:
        output.add(folder, arcname=stem)
    source = Path(os.environ['SOURCE_PACKAGE'])
    (delivery / 'SHA256SUMS').write_text(''.join(identity(path)['sha256'] + '  ' + path.name + '\n' for path in (archive, source)))
    print(json.dumps(provenance, indent=2, sort_keys=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('source', 'smoke', 'bundle'))
    parser.add_argument('--binary', type=Path)
    args = parser.parse_args()
    if args.action == 'source':
        source()
    elif args.action == 'smoke':
        smoke(args.binary)
    else:
        bundle(args.binary)


if __name__ == '__main__':
    main()
