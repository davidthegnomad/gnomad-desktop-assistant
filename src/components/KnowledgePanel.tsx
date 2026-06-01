import { useCallback, useEffect, useState } from "react";
import { Plus, Trash2, FolderOpen, FileText } from "lucide-react";
import {
  listKnowledgeFiles,
  pickAndImportKnowledge,
  deleteKnowledgeFile,
  getKnowledgeRoot,
  type KnowledgeCategory,
  type KnowledgeFileEntry,
} from "../lib/knowledge";
import { MUSHROOM, LLAMA } from "../lib/brand";

const CATEGORIES: { id: KnowledgeCategory; label: string; emoji: string }[] = [
  { id: "skills", label: "Skills", emoji: MUSHROOM },
  { id: "agents", label: "Agent knowledge", emoji: "🤖" },
  { id: "uploads", label: "Uploads", emoji: "📎" },
  { id: "preferences", label: "Preferences", emoji: LLAMA },
];

export function KnowledgePanel() {
  const [category, setCategory] = useState<KnowledgeCategory>("skills");
  const [files, setFiles] = useState<KnowledgeFileEntry[]>([]);
  const [root, setRoot] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const refresh = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const [list, path] = await Promise.all([
        listKnowledgeFiles(category),
        getKnowledgeRoot(),
      ]);
      setFiles(list);
      setRoot(path);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [category]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleAdd = async () => {
    setError("");
    try {
      await pickAndImportKnowledge(category);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteKnowledgeFile(id);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="knowledge-panel">
      <p className="knowledge-intro">
        Add skills, agent briefs, and reference files. Gnomad agents update{" "}
        <code>INDEX.md</code> and <code>user-preferences.md</code> as you work.
      </p>

      <div className="knowledge-tabs">
        {CATEGORIES.map((c) => (
          <button
            key={c.id}
            type="button"
            className={`knowledge-tab ${category === c.id ? "active" : ""}`}
            onClick={() => setCategory(c.id)}
          >
            <span aria-hidden>{c.emoji}</span> {c.label}
          </button>
        ))}
      </div>

      <div className="knowledge-toolbar">
        <button type="button" className="btn primary knowledge-add-btn" onClick={handleAdd}>
          <Plus size={16} />
          Add files
        </button>
        {root && (
          <span className="knowledge-path" title={root}>
            <FolderOpen size={12} />
            {root.replace(/^.*\/gnomad\//, "gnomad/")}
          </span>
        )}
      </div>

      {error && <p className="onboarding-error">{error}</p>}
      {loading && <p className="knowledge-muted">Loading…</p>}

      <ul className="knowledge-list">
        {files.length === 0 && !loading && (
          <li className="knowledge-empty">
            No files yet — tap <strong>Add files</strong> to upload `.md`, `.txt`, or `.json`.
          </li>
        )}
        {files.map((f) => (
          <li key={f.id} className="knowledge-item">
            <FileText size={14} className="knowledge-item-icon" />
            <div className="knowledge-item-meta">
              <span className="knowledge-item-name">{f.name}</span>
              <span className="knowledge-muted">
                {(f.size_bytes / 1024).toFixed(1)} KB · {f.category}
              </span>
            </div>
            <button
              type="button"
              className="icon-btn"
              title="Remove"
              onClick={() => handleDelete(f.id)}
            >
              <Trash2 size={14} />
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
