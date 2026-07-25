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
      <div className="mb-5 flex w-fit rounded-[3px] border border-[#3b3b40] bg-[#18181a] p-0.5">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActive(tab.id)}
            className={cn(
              'rounded-[2px] px-4 py-1.5 text-[12px] font-medium cursor-default',
              'transition duration-150',
              active === tab.id
                ? 'bg-[#3a3a3f] text-white shadow-[0_1px_3px_rgba(0,0,0,0.2)]'
                : 'text-[#8e8e93] hover:text-[#d1d1d6]',
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <div className="flex-1 min-h-0 overflow-auto">{current?.content}</div>
    </div>
  );
}
