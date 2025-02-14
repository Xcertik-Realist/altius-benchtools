import json

# file = r"./data/m2m-10k-70%-erc20.json"
file = r"./data/m2o-10k-100g-erc20.json"

ERC20_BYTECODE = open("./scripts/bytecode/erc20.bytecode").read()

for (bytecode_file, slot_num) in [
    ("./scripts/bytecode/slot8.bytecode", 8),
    ("./scripts/bytecode/slot16.bytecode", 16),
    ("./scripts/bytecode/slot32.bytecode", 32),
    ("./scripts/bytecode/slot64.bytecode", 64),
    ("./scripts/bytecode/slot128.bytecode", 128),
    ("./scripts/bytecode/slot256.bytecode", 256),
]:
    origin = json.load(open(file))
    ERC20P_BYTECODE = open(bytecode_file).read()
    code_flag = False
    for addr, prestate in origin['just-test']['pre'].items():
        if prestate['code'] == ERC20_BYTECODE:
            if code_flag:
                raise Exception("More than one ERC20 contract found")
            code_flag = True
            prestate['code'] = ERC20P_BYTECODE
            storage_keys = list(prestate['storage'].keys())
            for storage_key in storage_keys:
                for i in range(1, slot_num):
                    new_storage_key = '0x' + hex(int(storage_key, 16) + i)[2:].rjust(64, '0')
                    prestate['storage'][new_storage_key] = prestate['storage'][storage_key]
            prestate['storage'] = json.loads(json.dumps(prestate['storage'], sort_keys=True))
            origin['just-test']['pre'][addr] = prestate
    json.dump(origin, open(file.replace("erc20", "erc20p-slot%d" % slot_num), "w"), indent=2)
