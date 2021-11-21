try:
    with open("ignores.url") as file:
        datums = file.read().split("\n")
except FileNotFoundError:
    print("no ignores file")
    datums = []

sodium = []

import sqlite3
sqliteConnection = sqlite3.connect(input("DB filename? "))
cursor = sqliteConnection.cursor()

print("Connected to the DB!")

a = cursor.execute(
            """SELECT * FROM messages
ORDER BY 1;"""
)
import re
for i in a:
    for j in re.split("(\n| )", i[3]):
        if j.startswith("https://") and j not in datums:
            datums.append(j)
            sodium.append(j)
            #print(j)
print(len(datums), "urls processed.")

print("Writing to file.")
with open("ignores.url","w+") as ignrs:
    hi = ""
    for ignore in datums:
        hi += ignore + "\n"
    ignrs.write(hi)
with open("urls.url","w+") as urls:
    hi = ""
    for url in sodium:
        hi += url + "\n"
    urls.write(hi)
print("Written data.")
print("Ignores stored in ignores.url")
print("Urls stored in urls.url")
print()

sqliteConnection.close()

print("Closed connection.")
