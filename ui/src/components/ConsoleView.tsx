import { useEffect, useRef, useState } from 'react';
import { cn } from '@/lib/utils';

interface ConsoleViewProps {
  lines: string[];
  className?: string;
}

export function ConsoleView({ lines, className }: ConsoleViewProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  useEffect(() => {
    if (autoScroll && ref.current) {
      ref.current.scrollTop = ref.current.scrollHeight;
    }
  }, [lines, autoScroll]);

  const handleScroll = () => {
    if (!ref.current) return;
    const { scrollTop, scrollHeight, clientHeight } = ref.current;
    setAutoScroll(scrollHeight - scrollTop - clientHeight < 40);
  };

  return (
    <div
      ref={ref}
      onScroll={handleScroll}
      className={cn(
        'h-full min-h-[200px] overflow-auto font-mono text-[11px] leading-relaxed',
        'border-2 border-[#1a1a1a]',
        'shadow-[inset_0_2px_4px_rgba(0,0,0,0.5)]',
        'bg-[#0a0a0a] p-3',
        className,
      )}
    >
      {lines.length === 0 ? (
        <div className="text-[#336633] italic">Ready. Launch a game to see output here.</div>
      ) : (
        lines.map((line, i) => (
          <div
            key={i}
            className={cn(
              'whitespace-pre-wrap break-all',
              line.startsWith('ERROR') || line.includes('error')
                ? 'text-[#ff4444]'
                : line.startsWith('err:') || line.includes('warn')
                  ? 'text-[#ccaa44]'
                  : 'text-[#33cc33]',
            )}
          >
            {line}
          </div>
        ))
      )}
    </div>
  );
}
