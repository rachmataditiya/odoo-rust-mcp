import React, { useEffect, useState } from 'react';
import { Plus, Edit2, Trash2, RefreshCw, FileText } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Card } from '../Card';
import { Button } from '../Button';
import { StatusMessage } from '../StatusMessage';
import { PromptForm } from '../PromptForm';
import type { PromptConfig } from '../../types';

export function PromptsTab() {
  const { load, save, status, loading } = useConfig('prompts');
  const [prompts, setPrompts] = useState<PromptConfig[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingPrompt, setEditingPrompt] = useState<PromptConfig | null>(null);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);

  useEffect(() => {
    loadPrompts();
  }, []);

  const loadPrompts = async () => {
    try {
      const data = await load() as PromptConfig[];
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
        const exists = prompts.some(p => p.name === prompt.name);
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

  const truncateContent = (content: string, maxLength: number = 150) => {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + '...';
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900">System Prompts</h2>
          <p className="mt-2 text-gray-600">
            Configure AI prompts and system instructions for the MCP server.
          </p>
        </div>
        <Button
          onClick={handleAdd}
          icon={<Plus size={18} />}
          variant="primary"
          disabled={loading}
        >
          Add Prompt
        </Button>
      </div>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">
            Configured Prompts ({prompts.length})
          </h3>
          <Button
            onClick={loadPrompts}
            loading={loading}
            icon={<RefreshCw size={16} />}
            variant="ghost"
            size="sm"
          >
            Refresh
          </Button>
        </div>

        {prompts.length === 0 ? (
          <div className="text-center py-16">
            <FileText className="mx-auto text-gray-400 mb-3" size={48} />
            <p className="text-gray-500 mb-4">No prompts configured</p>
            <Button onClick={handleAdd} icon={<Plus size={16} />} variant="primary">
              Add Your First Prompt
            </Button>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-200 bg-gray-50">
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    Name
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    Description
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    Content Preview
                  </th>
                  <th className="px-4 py-3 text-right text-sm font-semibold text-gray-700">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200">
                {prompts.map((prompt, index) => (
                  <tr key={index} className="hover:bg-gray-50 transition-colors">
                    <td className="px-4 py-4">
                      <div className="flex items-center gap-2">
                        <FileText size={16} className="text-blue-600 flex-shrink-0" />
                        <code className="font-mono text-sm font-medium text-gray-900">
                          {prompt.name}
                        </code>
                      </div>
                    </td>
                    <td className="px-4 py-4">
                      <p className="text-sm text-gray-600 max-w-xs">
                        {prompt.description || '-'}
                      </p>
                    </td>
                    <td className="px-4 py-4">
                      <p className="text-xs text-gray-500 font-mono max-w-md line-clamp-2">
                        {truncateContent(prompt.content)}
                      </p>
                    </td>
                    <td className="px-4 py-4">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleEdit(prompt, index)}
                          className="p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                          title="Edit"
                        >
                          <Edit2 size={16} />
                        </button>
                        <button
                          onClick={() => handleDelete(index)}
                          className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                          title="Delete"
                        >
                          <Trash2 size={16} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {showForm && (
        <PromptForm
          prompt={editingPrompt}
          onSave={handleSavePrompt}
          onCancel={() => {
            setShowForm(false);
            setEditingPrompt(null);
            setEditingIndex(null);
          }}
        />
      )}
    </div>
  );
}
