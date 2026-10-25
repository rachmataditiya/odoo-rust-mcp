import React, { useState, useEffect } from 'react';
import { Button } from './Button';
import { Field } from './Field';
import { TextField } from './TextField';
import type { PromptConfig } from '../types';

export interface PromptFormProps {
  prompt: PromptConfig | null;
  onSave: (prompt: PromptConfig) => void;
  onCancel: () => void;
}

export function PromptForm({ prompt, onSave, onCancel }: PromptFormProps) {
  const [formData, setFormData] = useState<PromptConfig>({
    name: '',
    description: '',
    content: '',
  });

  const [errors, setErrors] = useState<{ name?: string; content?: string }>({});

  useEffect(() => {
    if (prompt) {
      setFormData(prompt);
    } else {
      setFormData({ name: '', description: '', content: '' });
    }
    setErrors({});
  }, [prompt]);

  const validate = () => {
    const newErrors: { name?: string; content?: string } = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Prompt identifier name is required';
    }

    if (!formData.content.trim()) {
      newErrors.content = 'Prompt template content is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) return;

    onSave({
      ...formData,
      name: formData.name.trim(),
      description: formData.description?.trim() || '',
      content: formData.content.trim(),
    });
  };

  return (
    <form onSubmit={handleSubmit} className="divide-y divide-line">
      {/* Prompt Name */}
      <Field
        label="Prompt Name"
        help="Identifier used by MCP clients to reference this prompt."
        htmlFor="prompt-name"
      >
        <div className="w-full sm:w-72">
          <TextField
            id="prompt-name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="e.g. odoo_common_models"
            mono
            disabled={!!prompt}
          />
          {errors.name && (
            <p className="mt-1 text-[11px] text-down font-medium">{errors.name}</p>
          )}
        </div>
      </Field>

      {/* Description */}
      <Field
        label="Description (Optional)"
        help="Summary of the prompt intent and expected outputs."
        htmlFor="prompt-desc"
      >
        <div className="w-full sm:w-72">
          <TextField
            id="prompt-desc"
            value={formData.description || ''}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            placeholder="Brief description of this prompt"
          />
        </div>
      </Field>

      {/* Prompt Content */}
      <div className="py-4 space-y-2">
        <div className="flex items-center justify-between">
          <label htmlFor="prompt-content" className="block text-[13px] font-medium text-fg">
            Prompt Template Content
          </label>
          <span className="text-[11px] text-muted">
            Supports parameter placeholders like <code className="text-primary font-mono">{`{{model}}`}</code>
          </span>
        </div>
        <textarea
          id="prompt-content"
          rows={7}
          value={formData.content}
          onChange={(e) => setFormData({ ...formData, content: e.target.value })}
          placeholder="Enter the prompt body or guidelines injected for the AI..."
          className="w-full rounded-md border border-line bg-surface-2 p-3 text-[13px] text-fg transition-colors outline-none placeholder:text-faint hover:border-line-strong focus:border-primary focus:ring-1 focus:ring-primary font-mono leading-relaxed"
        />
        {errors.content && (
          <p className="text-[11px] text-down font-medium">{errors.content}</p>
        )}
      </div>

      {/* Action buttons */}
      <div className="flex items-center justify-end gap-2.5 pt-4">
        <Button type="button" onClick={onCancel} variant="plain" size="md">
          Cancel
        </Button>
        <Button type="submit" variant="primary" size="md">
          {prompt ? 'Update Prompt' : 'Create Prompt'}
        </Button>
      </div>
    </form>
  );
}
