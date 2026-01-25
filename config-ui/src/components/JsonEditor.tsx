import React, { useEffect, useRef } from 'react';
import JSONEditor, { type JSONEditorOptions } from 'jsoneditor';
import 'jsoneditor/dist/jsoneditor.min.css';

interface JsonEditorProps {
  value: any;
  onChange: (value: any) => void;
  mode?: 'tree' | 'code' | 'view' | 'form' | 'text';
}

export const JsonEditor: React.FC<JsonEditorProps> = ({ 
  value, 
  onChange, 
  mode = 'tree' 
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<JSONEditor | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const options: JSONEditorOptions = {
      mode,
      modes: ['code', 'tree', 'view', 'form', 'text'],
      onError: (err) => {
        console.error('JSONEditor error:', err);
      },
      onModeChange: (newMode) => {
        console.log('Mode changed to:', newMode);
      },
      onChange: () => {
        try {
          const json = editorRef.current?.get();
          if (json !== undefined) {
            onChange(json);
          }
        } catch (e) {
          console.error('Error getting JSON from editor:', e);
        }
      },
    };

    editorRef.current = new JSONEditor(containerRef.current, options);

    return () => {
      if (editorRef.current) {
        editorRef.current.destroy();
        editorRef.current = null;
      }
    };
  }, [mode, onChange]);

  useEffect(() => {
    if (editorRef.current && value !== undefined) {
      try {
        editorRef.current.set(value);
      } catch (e) {
        console.error('Error setting JSON in editor:', e);
      }
    }
  }, [value]);

  return (
    <div 
      ref={containerRef} 
      className="w-full border border-gray-300 rounded"
      style={{ height: '600px' }}
    />
  );
};
