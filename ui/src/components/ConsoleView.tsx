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
        'h-full min-h-[200px] overflow-auto rounded-[3px] font-mono text-[11px] leading-relaxed',
        'border border-[#38383c] bg-[#0d0d0f] p-4',
        'shadow-[inset_0_1px_8px_rgba(0,0,0,0.3)]',
        'selectable-diagnostic',
        className,
      )}
    >
      {lines.length === 0 ? (
        <div className="italic text-white/30">Ready. Launch a game to see output here.</div>
      ) : (
        lines.map((line, i) => (
          <div
            key={i}
            className={cn(
              'whitespace-pre-wrap break-all',
              line.startsWith('ERROR') || line.includes('error')
                ? 'text-[#ff6961]'
                : line.startsWith('err:') || line.includes('warn')
                  ? 'text-[#ffc15c]'
                  : 'text-[#7ce997]',
            )}
          >
            {line}
          </div>
        ))
      )}
    </div>
  );
}
