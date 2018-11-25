def group(name: str, ary: list):
    i = 1
    while ary:
        with open(f'{name}_{i}.txt', 'w') as w:
            i += 1
            for _ in range(40):
                if ary:
                    w.write(f'{ary.pop()}\n')
                else:
                    break

d = {}
with open('orders.txt', 'r') as r:
    for l in r:
        kind, account = l.split('-')
        account = account.strip()
        if kind not in d:
            d[kind] = [account]
        else:
            d[kind].append(account)
            
for k in d:
    if k.startswith('1'):
        group('季', d[k])
    elif k.startswith('6'):
        group('月', d[k])
    elif k.startswith('2'): 
        group('周', d[k])
    else:
        group('未知', d[k])

