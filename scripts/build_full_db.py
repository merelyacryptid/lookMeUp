import sqlite3
import json
import gzip
import time
import os
import sys

def build_full_database(jsonl_path, db_path="dictionary.db", batch_size=20000):
    start_time = time.time()

    # Remove existing db to start fresh
    if os.path.exists(db_path):
        os.remove(db_path)

    conn = sqlite3.connect(db_path)
    cur = conn.cursor()

    # High-throughput ingestion pragmas
    cur.execute("PRAGMA synchronous = OFF;")
    cur.execute("PRAGMA journal_mode = MEMORY;")
    cur.execute("PRAGMA cache_size = 100000;")

    # 1. Create bare tables WITHOUT indexes
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

    # Open plain or gzipped file transparently
    opener = gzip.open if jsonl_path.endswith(".gz") else open
    
    entries_batch = []
    forms_batch = []
    total_entries = 0

    print(f"Ingesting from: {jsonl_path}")

    with opener(jsonl_path, "rt", encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            try:
                data = json.loads(line)
            except Exception:
                continue

            word = data.get("word")
            pos = data.get("pos", "")
            if not word:
                continue

            # Extract glosses/meanings
            meanings = []
            for sense in data.get("senses", []):
                for gloss in sense.get("glosses", []):
                    if gloss:
                        meanings.append(gloss)

            if not meanings:
                continue

            entries_batch.append((word, pos, json.dumps(meanings)))

            # Extract inflected forms (form_of mapping)
            for form_info in data.get("forms", []):
                form_str = form_info.get("form")
                if form_str and form_str.lower() != word.lower():
                    forms_batch.append((form_str, word))

            total_entries += 1

            # Flush batch to SQLite
            if len(entries_batch) >= batch_size:
                cur.executemany("INSERT INTO entries (word, part_of_speech, meanings) VALUES (?, ?, ?)", entries_batch)
                cur.executemany("INSERT INTO forms (form, headword) VALUES (?, ?)", forms_batch)
                conn.commit()
                entries_batch.clear()
                forms_batch.clear()
                print(f"Processed {total_entries} words...")

    # Flush remainder
    if entries_batch:
        cur.executemany("INSERT INTO entries (word, part_of_speech, meanings) VALUES (?, ?, ?)", entries_batch)
        cur.executemany("INSERT INTO forms (form, headword) VALUES (?, ?)", forms_batch)
        conn.commit()

    print(f"Inserted {total_entries} words. Now building B-tree indexes...")
    index_start = time.time()
    
    # 2. Build B-tree indexes at the very end
    cur.execute("CREATE INDEX idx_entries_word ON entries(word COLLATE NOCASE);")
    cur.execute("CREATE INDEX idx_forms_form ON forms(form COLLATE NOCASE);")
    conn.commit()
    conn.close()

    total_time = time.time() - start_time
    print(f"Database build complete in {total_time:.2f}s (Index build: {time.time() - index_start:.2f}s)!")

if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "scripts/sample_kaikki.jsonl"
    build_full_database(path)
