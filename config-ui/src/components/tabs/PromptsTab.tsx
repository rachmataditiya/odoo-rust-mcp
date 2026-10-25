import { useEffect, useState } from 'react';
import { Plus, Edit2, Trash2, FileText, Copy, Check } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Button } from '../Button';
import { StatusMessage } from '../StatusMessage';
import { PromptForm } from '../PromptForm';
import { Modal } from '../Modal';
import { PageHeader } from '../PageHeader';
import type { PromptConfig } from '../../types';

export function PromptsTab() {
  const { load, save, status, loading } = useConfig('prompts');
  const [prompts, setPrompts] = useState<PromptConfig[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingPrompt, setEditingPrompt] = useState<PromptConfig | null>(null);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [copiedName, setCopiedName] = useState<string | null>(null);

  useEffect(() => {
    loadPrompts();
  }, []);

  const loadPrompts = async () => {
    try {
      const data = (await load()) as PromptConfig[];
      setPrompts(data);
    } catch (error) {
      console.error('Failed to load prompts:', error);
    }
  };

  const handleAdd = () => {
    setEditingPrompt(null);
    setEditingIndex(null);
    setShowForm(true);
  };

  const handleEdit = (prompt: PromptConfig, index: number) => {
    setEditingPrompt(prompt);
    setEditingIndex(index);
    setShowForm(true);
  };

  const handleDelete = async (index: number) => {
    if (!confirm('Are you sure you want to delete this prompt?')) {
      return;
    }

    try {
      const updatedPrompts = prompts.filter((_, i) => i !== index);
      await save(updatedPrompts);
      await loadPrompts();
    } catch (error) {
      console.error('Failed to delete prompt:', error);
    }
  };

  const handleSavePrompt = async (prompt: PromptConfig) => {
    try {
      let updatedPrompts: PromptConfig[];

      if (editingIndex !== null) {
        updatedPrompts = prompts.map((p, i) => (i === editingIndex ? prompt : p));
      } else {
        const exists = prompts.some((p) => p.name === prompt.name);
        if (exists) {
          alert('A prompt with this name already exists. Please use a different name.');
          return;
        }
        updatedPrompts = [...prompts, prompt];
      }

      await save(updatedPrompts);
      await loadPrompts();
      setShowForm(false);
      setEditingPrompt(null);
      setEditingIndex(null);
    } catch (error) {
      console.error('Failed to save prompt:', error);
    }
  };

  const copyPromptContent = async (name: string, content: string) => {
    await navigator.clipboard.writeText(content);
    setCopiedName(name);
    setTimeout(() => setCopiedName(null), 2000);
  };

  // Helper to extract {{var}} placeholders from prompt content
  const extractPlaceholders = (content: string) => {
    const matches = content.match(/\{\{([^}]+)\}\}/g);
    if (!matches) return [];
    return Array.from(new Set(matches.map((m) => m.replace(/[{}]/g, '').trim())));
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="System Prompts"
        description="Define reusable prompt templates exposed via the MCP prompts protocol."
      >
        <Button
          onClick={handleAdd}
          icon={<Plus size={15} />}
          variant="primary"
          size="md"
          disabled={loading}
        >
          Add Prompt
        </Button>
      </PageHeader>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      {prompts.length === 0 ? (
        <div className="rounded-xl border border-dashed border-line bg-surface/50 p-12 text-center">
          <div className="mx-auto w-12 h-12 rounded-xl bg-surface-2 border border-line flex items-center justify-center text-muted mb-4 shadow-xs">
            <FileText size={22} />
          </div>
          <h3 className="text-[14px] font-semibold text-fg">No Prompts Configured</h3>
          <p className="mt-1 text-[13px] text-muted max-w-sm mx-auto">
            Create AI prompts such as domain search helpers or model relationship inspectors.
          </p>
          <div className="mt-5">
            <Button onClick={handleAdd} icon={<Plus size={15} />} variant="primary" size="md">
              Add First Prompt
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          {prompts.map((prompt, index) => {
            const placeholders = extractPlaceholders(prompt.content);
            const isCopied = copiedName === prompt.name;

            return (
              <div
                key={prompt.name}
                className="overflow-hidden rounded-xl border border-line bg-surface shadow-sm hover:border-line-strong transition-all"
              >
                {/* Header */}
                <div className="flex items-center justify-between gap-3 border-b border-line bg-surface-2/40 px-5 py-3.5">
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="w-7 h-7 rounded-lg bg-primary/10 text-primary flex items-center justify-center shrink-0">
                      <FileText size={15} />
                    </div>
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-[13px] font-semibold text-fg truncate">
                          {prompt.name}
                        </span>
                        {placeholders.length > 0 && (
                          <div className="hidden sm:flex items-center gap-1">
                            {placeholders.slice(0, 3).map((p) => (
                              <span
                                key={p}
                                className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-primary/10 text-primary border border-primary/20"
                              >
                                {`{{${p}}}`}
                              </span>
                            ))}
                            {placeholders.length > 3 && (
                              <span className="text-[10px] text-muted font-mono">
                                +{placeholders.length - 3}
                              </span>
                            )}
                          </div>
                        )}
                      </div>
                      {prompt.description && (
                        <p className="mt-0.5 text-[12px] text-muted truncate">
                          {prompt.description}
                        </p>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0">
                    <button
                      type="button"
                      onClick={() => copyPromptContent(prompt.name, prompt.content)}
                      className="grid h-7 w-7 place-items-center rounded-md text-muted hover:text-fg hover:bg-surface-3 transition-colors"
                      title={isCopied ? 'Copied to clipboard' : 'Copy prompt template'}
                      aria-label="Copy prompt template"
                    >
                      {isCopied ? <Check size={13} className="text-ok" /> : <Copy size={13} />}
                    </button>
                    <button
                      type="button"
                      onClick={() => handleEdit(prompt, index)}
                      className="grid h-7 w-7 place-items-center rounded-md text-muted hover:text-fg hover:bg-surface-3 transition-colors"
                      title="Edit prompt"
                      aria-label="Edit prompt"
                    >
                      <Edit2 size={13} />
                    </button>
                    <button
                      type="button"
                      onClick={() => handleDelete(index)}
                      className="grid h-7 w-7 place-items-center rounded-md text-muted hover:text-down hover:bg-down/10 transition-colors"
                      title="Delete prompt"
                      aria-label="Delete prompt"
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                </div>

                {/* Content Body */}
                <div className="p-4 bg-surface-2/20">
                  <pre className="font-mono text-[12px] text-fg/90 bg-surface rounded-lg p-3.5 border border-line whitespace-pre-wrap break-words leading-relaxed max-h-48 overflow-y-auto">
                    {prompt.content}
                  </pre>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Add / Edit Prompt Modal */}
      <Modal
        isOpen={showForm}
        onClose={() => {
          setShowForm(false);
          setEditingPrompt(null);
          setEditingIndex(null);
        }}
        title={editingPrompt ? `Edit "${editingPrompt.name}"` : 'New System Prompt'}
        description="Configure an MCP prompt template for client models."
      >
        <PromptForm
          prompt={editingPrompt}
          onSave={handleSavePrompt}
          onCancel={() => {
            setShowForm(false);
            setEditingPrompt(null);
            setEditingIndex(null);
          }}
        />
      </Modal>
    </div>
  );
}
