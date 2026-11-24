import sqlite3
import json
import gzip
import time
import os
import sys
import urllib.request

DATASET_URL = "https://raw.githubusercontent.com/CloudBytes-Academy/English-Dictionary-Open-Source/main/v2/dictionary.jsonl.gz"
LOCAL_DATASET = "scripts/dictionary.jsonl.gz"

IRREGULAR_FORMS = {
    # Verbs
    "ran": "run", "running": "run", "runs": "run",
    "went": "go", "gone": "go", "going": "go", "goes": "go",
    "saw": "see", "seen": "see", "seeing": "see", "sees": "see",
    "had": "have", "has": "have", "having": "have",
    "was": "be", "were": "be", "been": "be", "being": "be", "is": "be", "are": "be", "am": "be",
    "took": "take", "taken": "take", "taking": "take", "takes": "take",
    "wrote": "write", "written": "write", "writing": "write", "writes": "write",
    "read": "read", "reading": "read", "reads": "read",
    "spoke": "speak", "spoken": "speak", "speaking": "speak", "speaks": "speak",
    "bought": "buy", "buying": "buy", "buys": "buy",
    "brought": "bring", "bringing": "bring", "brings": "bring",
    "thought": "think", "thinking": "think", "thinks": "think",
    "caught": "catch", "catching": "catch", "catches": "catch",
    "taught": "teach", "teaching": "teach", "teaches": "teach",
    "felt": "feel", "feeling": "feel", "feels": "feel",
    "found": "find", "finding": "find", "finds": "find",
    "gave": "give", "given": "give", "giving": "give", "gives": "give",
    "knew": "know", "known": "know", "knowing": "know", "knows": "know",
    "lost": "lose", "losing": "lose", "loses": "lose",
    "made": "make", "making": "make", "makes": "make",
    "met": "meet", "meeting": "meet", "meets": "meet",
    "paid": "pay", "paying": "pay", "pays": "pay",
    "said": "say", "saying": "say", "says": "say",
    "sent": "send", "sending": "send", "sends": "send",
    "sat": "sit", "sitting": "sit", "sits": "sit",
    "stood": "stand", "standing": "stand", "stands": "stand",
    "told": "tell", "telling": "tell", "tells": "tell",
    "understood": "understand", "understanding": "understand", "understands": "understand",
    "won": "win", "winning": "win", "wins": "win",
    "began": "begin", "begun": "begin", "beginning": "begin", "begins": "begin",
    "chose": "choose", "chosen": "choose", "choosing": "choose", "chooses": "choose",
    "drew": "draw", "drawn": "draw", "drawing": "draw", "draws": "draw",
    "drank": "drink", "drunk": "drink", "drinking": "drink", "drinks": "drink",
    "drove": "drive", "driven": "drive", "driving": "drive", "drives": "drive",
    "ate": "eat", "eaten": "eat", "eating": "eat", "eats": "eat",
    "fell": "fall", "fallen": "fall", "falling": "fall", "falls": "fall",
    "flew": "fly", "flown": "fly", "flying": "fly", "flies": "fly",
    "forgot": "forget", "forgotten": "forget", "forgetting": "forget", "forgets": "forget",
    "froze": "freeze", "frozen": "freeze", "freezing": "freeze", "freezes": "freeze",
    "grew": "grow", "grown": "grow", "growing": "grow", "grows": "grow",
    "hid": "hide", "hidden": "hide", "hiding": "hide", "hides": "hide",
    "rode": "ride", "ridden": "ride", "riding": "ride", "rides": "ride",
    "rang": "ring", "rung": "ring", "ringing": "ring", "rings": "ring",
    "rose": "rise", "risen": "rise", "rising": "rise", "rises": "rise",
    "shook": "shake", "shaken": "shake", "shaking": "shake", "shakes": "shake",
    "shone": "shine", "shining": "shine", "shines": "shine",
    "sang": "sing", "sung": "sing", "singing": "sing", "sings": "sing",
    "sunk": "sink", "sunken": "sink", "sinking": "sink", "sinks": "sink",
    "slept": "sleep", "sleeping": "sleep", "sleeps": "sleep",
    "slid": "slide", "sliding": "slide", "slides": "slide",
    "swam": "swim", "swum": "swim", "swimming": "swim", "swims": "swim",
    "threw": "throw", "thrown": "throw", "throwing": "throw", "throws": "throw",
    "woke": "wake", "woken": "wake", "waking": "wake", "wakes": "wake",
    "wore": "wear", "worn": "wear", "wearing": "wear", "wears": "wear",
    # Nouns
    "children": "child", "men": "man", "women": "woman",
    "feet": "foot", "teeth": "tooth", "mice": "mouse", "geese": "goose",
    "people": "person", "leaves": "leaf", "halves": "half",
    "wives": "wife", "lives": "life", "wolves": "wolf",
    "calves": "calf", "knives": "knife", "analyses": "analysis",
    "crises": "crisis", "theses": "thesis", "criteria": "criterion",
}

def generate_regular_forms(word, pos_set):
    forms = set()
    w = word.lower()
    if not w.isalpha() or len(w) < 3:
        return forms

    # 1. Plural / 3rd person singular (-s, -es, -ies)
    if w.endswith(('s', 'x', 'z', 'ch', 'sh')):
        forms.add(w + 'es')
    elif w.endswith('y') and len(w) > 2 and w[-2] not in 'aeiou':
        forms.add(w[:-1] + 'ies')
    else:
        forms.add(w + 's')

    # 2. Past tense & participle (-ed, -d, -ied)
    if "verb" in pos_set or not pos_set:
        if w.endswith('e'):
            forms.add(w + 'd')
        elif w.endswith('y') and len(w) > 2 and w[-2] not in 'aeiou':
            forms.add(w[:-1] + 'ied')
        else:
            forms.add(w + 'ed')
            if len(w) >= 3 and w[-1] not in 'aeiouwxy' and w[-2] in 'aeiou' and w[-3] not in 'aeiou':
                forms.add(w + w[-1] + 'ed')

    # 3. Present participle (-ing)
    if "verb" in pos_set or not pos_set:
        if w.endswith('ie'):
            forms.add(w[:-2] + 'ying')
        elif w.endswith('e') and not w.endswith('ee'):
            forms.add(w[:-1] + 'ing')
        else:
            forms.add(w + 'ing')
            if len(w) >= 3 and w[-1] not in 'aeiouwxy' and w[-2] in 'aeiou' and w[-3] not in 'aeiou':
                forms.add(w + w[-1] + 'ing')

    return forms

def ensure_dataset():
    if not os.path.exists(LOCAL_DATASET):
        print(f"Downloading dataset from {DATASET_URL}...")
        t0 = time.time()
        urllib.request.urlretrieve(DATASET_URL, LOCAL_DATASET)
        print(f"Downloaded in {time.time() - t0:.2f}s!")

def build_full_database(dataset_path=LOCAL_DATASET, db_path="dictionary.db", batch_size=25000):
    start_time = time.time()
    ensure_dataset()

    tmp_db_path = db_path + ".tmp"
    if os.path.exists(tmp_db_path):
        os.remove(tmp_db_path)

    conn = sqlite3.connect(tmp_db_path)
    cur = conn.cursor()

    # High-throughput SQLite bulk ingestion pragmas
    cur.execute("PRAGMA synchronous = OFF;")
    cur.execute("PRAGMA journal_mode = MEMORY;")
    cur.execute("PRAGMA cache_size = 100000;")
    cur.execute("PRAGMA temp_store = MEMORY;")

    # Create bare tables without indexes for high-speed batch insertion
    cur.execute("""
        CREATE TABLE entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            word TEXT NOT NULL,
            part_of_speech TEXT NOT NULL,
            meanings TEXT NOT NULL
        );
    """)
    cur.execute("""
        CREATE TABLE forms (
            form TEXT NOT NULL,
            headword TEXT NOT NULL
        );
    """)

    print(f"Parsing and aggregating dictionary data from: {dataset_path}...")
    opener = gzip.open if dataset_path.endswith(".gz") else open

    # Aggregate definitions and parts of speech by headword
    # word -> {'pos': set(), 'meanings': list, 'forms': set()}
    words_map = {}
    total_raw_records = 0

    with opener(dataset_path, "rt", encoding="utf-8") as f:
        for line in f:
            total_raw_records += 1
            if total_raw_records % 100000 == 0:
                print(f"  Processed {total_raw_records:,} raw records...")
            try:
                data = json.loads(line)
            except Exception:
                continue

            word = data.get("word", "").strip()
            if not word:
                continue

            pos = (data.get("pos") or data.get("wordtype") or "").strip().lower()

            # Extract definitions (handles both CloudBytes and Kaikki formats)
            defn_list = []
            if "definition" in data and data["definition"]:
                defn_list.append(data["definition"].strip())
            if "senses" in data:
                for sense in data["senses"]:
                    for gloss in sense.get("glosses", []):
                        if gloss:
                            defn_list.append(gloss.strip())

            if not defn_list:
                continue

            if word not in words_map:
                words_map[word] = {"pos": set(), "meanings": [], "forms": set()}

            if pos:
                words_map[word]["pos"].add(pos)

            for d in defn_list:
                if d and d not in words_map[word]["meanings"]:
                    words_map[word]["meanings"].append(d)

            # Kaikki explicit forms if present
            for form_info in data.get("forms", []):
                f_str = form_info.get("form")
                if f_str and f_str.lower() != word.lower():
                    words_map[word]["forms"].add(f_str.lower())

    print(f"Aggregated {len(words_map):,} unique headwords from {total_raw_records:,} raw records.")
    print("Populating 'entries' table in SQLite...")

    entries_batch = []
    forms_set = set() # (form, headword) pairs

    # Also add standard irregular forms
    for form_variant, hw in IRREGULAR_FORMS.items():
        forms_set.add((form_variant.lower(), hw))

    for word, item in words_map.items():
        pos_str = ", ".join(sorted(item["pos"])) or "unknown"
        meanings_json = json.dumps(item["meanings"])
        entries_batch.append((word, pos_str, meanings_json))

        # Explicit forms from Kaikki
        for f in item["forms"]:
            forms_set.add((f, word))

        # Generated regular inflected forms
        generated = generate_regular_forms(word, item["pos"])
        for gen_f in generated:
            if gen_f.lower() != word.lower():
                forms_set.add((gen_f.lower(), word))

        if len(entries_batch) >= batch_size:
            cur.executemany("INSERT INTO entries (word, part_of_speech, meanings) VALUES (?, ?, ?)", entries_batch)
            conn.commit()
            entries_batch.clear()

    if entries_batch:
        cur.executemany("INSERT INTO entries (word, part_of_speech, meanings) VALUES (?, ?, ?)", entries_batch)
        conn.commit()
        entries_batch.clear()

    print(f"Populated entries. Now populating {len(forms_set):,} morphological mappings in 'forms' table...")
    forms_list = list(forms_set)
    for i in range(0, len(forms_list), batch_size):
        chunk = forms_list[i : i + batch_size]
        cur.executemany("INSERT INTO forms (form, headword) VALUES (?, ?)", chunk)
        conn.commit()

    print("Building B-tree indexes on 'entries(word COLLATE NOCASE)' and 'forms(form COLLATE NOCASE)'...")
    index_start = time.time()
    cur.execute("CREATE INDEX idx_entries_word ON entries(word COLLATE NOCASE);")
    cur.execute("CREATE INDEX idx_forms_form ON forms(form COLLATE NOCASE);")
    conn.commit()
    print(f"Index creation completed in {time.time() - index_start:.2f}s.")

    conn.close()

    # Atomically replace destination database
    if os.path.exists(db_path):
        os.remove(db_path)
    os.rename(tmp_db_path, db_path)

    total_time = time.time() - start_time
    final_size_mb = os.path.getsize(db_path) / (1024 * 1024)
    print(f"Database build complete in {total_time:.2f}s!")
    print(f"Final database: {db_path} ({final_size_mb:.2f} MB)")

    # Run quick verification test
    verify_database(db_path)

def verify_database(db_path="dictionary.db"):
    print("\n--- Verifying Database Performance & Accuracy ---")
    conn = sqlite3.connect(db_path)
    cur = conn.cursor()

    cur.execute("SELECT COUNT(*) FROM entries;")
    entry_count = cur.fetchone()[0]
    cur.execute("SELECT COUNT(*) FROM forms;")
    form_count = cur.fetchone()[0]

    print(f"Total entries: {entry_count:,}")
    print(f"Total morphological forms: {form_count:,}")

    test_queries = [
        ("score", True),
        ("scored", False),
        ("epiphany", True),
        ("epiphanies", False),
        ("serendipity", True),
        ("ubiquitous", True),
        ("computer", True),
        ("running", False),
        ("ran", False),
        ("children", False),
    ]

    for word, is_headword in test_queries:
        t0 = time.perf_counter()
        cur.execute("SELECT word, part_of_speech, meanings FROM entries WHERE word = ? COLLATE NOCASE", (word,))
        row = cur.fetchone()
        if not row:
            # Fallback to forms table
            cur.execute("SELECT headword FROM forms WHERE form = ? COLLATE NOCASE", (word,))
            hw_row = cur.fetchone()
            if hw_row:
                cur.execute("SELECT word, part_of_speech, meanings FROM entries WHERE word = ? COLLATE NOCASE", (hw_row[0],))
                row = cur.fetchone()

        elapsed_ms = (time.perf_counter() - t0) * 1000.0
        if row:
            meanings = json.loads(row[2])
            print(f"Lookup '{word}' -> '{row[0]}' [{row[1]}] ({len(meanings)} meanings) in {elapsed_ms:.3f} ms")
        else:
            print(f"Lookup '{word}' -> NOT FOUND in {elapsed_ms:.3f} ms")

    conn.close()

if __name__ == "__main__":
    target_path = sys.argv[1] if len(sys.argv) > 1 else LOCAL_DATASET
    build_full_database(target_path)
