import json
import argparse
import graphviz

truncate = 6

parser = argparse.ArgumentParser(description='Visualize transaction flow')
parser.add_argument('data_path', type=str, help='Path to the JSON data file')
args = parser.parse_args()

data_path = args.data_path

with open(data_path, 'r') as f:
    data = json.load(f)

dot = graphviz.Digraph(comment='Transaction Flow')

dot.attr(ratio='auto')
dot.attr('node', shape='circle', fillcolor='lightblue')

pre = data['just-test']['pre']
transactions = data['just-test']['transaction']
addresses = set()

for idx, tx in enumerate(transactions):
    sender = tx['sender'][:truncate+2]
    sender_balance = int(pre[tx['sender']]['balance'], 16) / 1e18 if tx['sender'] in pre else 0
    if sender not in addresses:
        dot.node(sender, f'{sender}\n({sender_balance:.2f}e)')
        addresses.add(sender)

    if tx['value'] != '0x00':
        receiver = tx['to'][:truncate+2]
        receiver_balance = int(pre[tx['to']]['balance'], 16) / 1e18 if tx['to'] in pre else 0
        if receiver not in addresses:
            dot.node(receiver, f'{receiver}\n({receiver_balance:.2f}e)')
            addresses.add(receiver)
        value_eth = int(tx['value'], 16) / 1e18
        dot.edge(sender, receiver, label=f'[Tx:{idx}] {value_eth:.2f}e')
        
    elif tx['data'].startswith('0xa9059cbb'):   # transfer(to, value)
        receiver = '0x' + tx['data'][2+8+24:2+8+64][:truncate]
        value_usdc = int(tx['data'][2+8+64:], 16) / 1e18
        dot.edge(sender, receiver, label=f'[Tx:{idx}] {value_usdc:.2f}u')

    elif tx['data'].startswith('0x1249c58b'):
        dot.edge(sender, sender, label=f'[Tx:{idx}] mint 1000000u')

dot.render(f'./visualize/{data_path.split("/")[-1].split(".")[0]}', format='png', cleanup=True)
