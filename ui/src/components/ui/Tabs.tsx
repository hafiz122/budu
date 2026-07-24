import { useState } from 'react';
import { cn } from '@/lib/utils';

interface Tab {
  id: string;
  label: string;
  content: React.ReactNode;
}

interface TabsProps {
  tabs: Tab[];
  defaultTab?: string;
  className?: string;
}

export function Tabs({ tabs, defaultTab, className }: TabsProps) {
  const [active, setActive] = useState(defaultTab ?? tabs[0]?.id ?? '');

  const current = tabs.find((t) => t.id === active);

  return (
    <div className={cn('flex flex-col h-full', className)}>
      <div className="flex border-b-2 border-[#2a2a2a] mb-3">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActive(tab.id)}
            className={cn(
              'px-4 py-1.5 text-[11px] font-bold uppercase tracking-wide cursor-default',
              'border-2 border-b-0 transition-none',
              active === tab.id
                ? cn(
                    'bg-[linear-gradient(180deg,#4a4a4a_0%,#383838_100%)]',
                    'text-[#e0e0d0] text-shadow',
                    'border-[#5a5a5a] border-t-[#6a6a6a]',
                    'shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]',
                    '-mb-[2px]',
                  )
                : cn(
                    'bg-[linear-gradient(180deg,#333_0%,#2a2a2a_100%)]',
                    'text-[#888]',
                    'border-transparent',
                    'hover:bg-[linear-gradient(180deg,#3a3a3a_0%,#303030_100%)] hover:text-[#aaa]',
                  ),
            )}
          >
            {tab.label}
          </button>
        ))}
        <div className="flex-1 border-b-2 border-[#2a2a2a]" />
      </div>
      <div className="flex-1 min-h-0 overflow-auto">{current?.content}</div>
    </div>
  );
}
