import re, sqlite3

try:
    with open("ignores.url") as file:
        datums = file.read().split("\n")
except FileNotFoundError:
    print("no ignores file, using empty igset")
    datums = []
sodium = []

sqliteConnection = sqlite3.connect(input("DB filename? "))
cursor = sqliteConnection.cursor()

print("Connected to the DB!")

# messages table is where the messages are (duh). They are stored in a tuple, the 4th value is the message data.
messages = cursor.execute(
            """SELECT * FROM messages
ORDER BY 1;"""
)
a = messages
cursor = sqliteConnection.cursor()
attachments = cursor.execute(
        """SELECT * FROM attachments
ORDER BY 1;"""
)
for i in messages:
    for j in re.split("(\n| |\(|\)|\<|\>)", i[3]): # Split at every newline, space, or parentheses
        if (j.startswith("https://") or j.startswith("http://")) and j not in datums:
            datums.append(j)
            sodium.append(j)
            #print(j)

for attach in attachments:
    attachm = attach[4] # Attachment URL is stored in the 5th item in the tuple.
    #print(attachm)
    datums.append(attachm)
    sodium.append(attachm)
print(len(sodium), "urls processed.")
print("Writing to file.")
with open("ignores.url","w+") as ignrs: # This file lists URLs that have already been extracted. This way you can do incremental runs
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
print("Urls stored in urls.url\n")

sqliteConnection.close()

print("Closed connection.")
