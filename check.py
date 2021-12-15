with open("olde/ignores.url") as file: i = file.read().split("\n")
with open("urls.url") as file: u = file.read().split("\n")

for j in u:
    if j not in i: print(j)
