import sqlite3
import json

SAMPLE_ENTRIES = [
    {
        "word": "score",
        "pos": "verb",
        "meanings": ["To gain points in a game or contest.", "To cut or scratch a mark into a surface."],
        "forms": ["scored", "scoring", "scores"]
    },
    {
        "word": "epiphany",
        "pos": "noun",
        "meanings": ["A sudden, intuitive perception of or insight into the reality or essential meaning of something."],
        "forms": ["epiphanies"]
    }
]

def build_db(db_path="dictionary.db"):
    conn = sqlite3.connect(db_path)
    cur = conn.cursor()

    cur.executescript("""
    CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY,
    word TEXT NOT NULL,
    part_of_speech TEXT NOT NULL,
    meanings TEXT NOT NULL -- JSON array of strings
);
CREATE INDEX IF NOT EXISTS idx_entries_word ON entries(word COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS forms (
    form TEXT NOT NULL,
    headword TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_forms_form ON forms(form COLLATE NOCASE);
    """)
    
    for entry in SAMPLE_ENTRIES:
        cur.execute(
            "INSERT INTO entries (word, part_of_speech, meanings) VALUES (?, ?, ?)",
            (entry["word"], entry["pos"], json.dumps(entry["meanings"]))
        )
        for form in entry.get("forms", []):
            cur.execute(
                "INSERT INTO forms (form, headword) VALUES (?, ?)",
                (form, entry["word"])
            )

    conn.commit()
    conn.close()
    print("Database built successfully.")


if __name__ == "__main__":
    build_db()