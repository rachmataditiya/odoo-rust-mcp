import { useEffect, useRef } from 'react';
import JSONEditor, { JSONEditorOptions } from 'jsoneditor';
import 'jsoneditor/dist/jsoneditor.css';

interface JsonEditorProps {
  value: any;
  onChange: (value: any) => void;
  mode?: 'tree' | 'code' | 'form' | 'text' | 'view';
}

export function JsonEditor({ value, onChange, mode = 'code' }: JsonEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<JSONEditor | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const options: JSONEditorOptions = {
      mode,
      modes: ['code', 'tree'],
      onChange: () => {
        try {
          if (editorRef.current) {
            const updatedJson = editorRef.current.get();
            onChange(updatedJson);
          }
        } catch (error) {
          console.error('Invalid JSON:', error);
        }
      },
      onError: (error) => {
        console.error('JSON Editor error:', error);
      },
    };

    editorRef.current = new JSONEditor(containerRef.current, options);
    editorRef.current.set(value);

    return () => {
      if (editorRef.current) {
        editorRef.current.destroy();
        editorRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    if (editorRef.current) {
      try {
        const currentJson = editorRef.current.get();
        if (JSON.stringify(currentJson) !== JSON.stringify(value)) {
          editorRef.current.update(value);
        }
      } catch {
        editorRef.current.set(value);
      }
    }
  }, [value]);

  return <div ref={containerRef} className="json-editor-container w-full h-[500px]" />;
}
