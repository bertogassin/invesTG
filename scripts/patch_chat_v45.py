import re, sys, shutil, datetime

PATH = "static/chat-v2.js"

def fail(msg):
    print("PATCH_ABORTED: " + msg)
    sys.exit(1)

with open(PATH, "r", encoding="utf-8") as f:
    src = f.read()
original_src = src

sig = re.compile(
    r'(requestJson\(\s*"/api/chat/" \+\s*otherUserId \+\s*"/messages/" \+\s*'
    r'selectedMessage\.id \+\s*"/delete",\s*\{ method: "POST" \}\s*\)\s*'
    r'\.then\(function \(data\) \{.*?closeDelete\(\);\s*\}\))\s*'
    r'(\.finally\(function \(\) \{)',
    re.DOTALL
)
matches = list(sig.finditer(src))
if len(matches) != 1:
    fail("delete handler anchor found %d times" % len(matches))

print("ANCHOR_OK")

m = matches[0]
insert = (
    m.group(1) +
    '\n                    .catch(function () {\n'
    '                        deleteApply.classList.add("is-error");\n'
    '                    })\n                    ' +
    m.group(2)
)
src = src[:m.start()] + insert + src[m.end():]

if src == original_src:
    fail("no changes produced")

backup = PATH + ".before-v45-" + datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
shutil.copyfile(PATH, backup)
with open(PATH, "w", encoding="utf-8") as f:
    f.write(src)

print("PATCH_OK")
print("BACKUP=" + backup)
