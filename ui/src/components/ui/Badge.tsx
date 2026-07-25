import { cn } from '@/lib/utils';

interface BadgeProps {
  children: React.ReactNode;
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
  className?: string;
}

const variants = {
  default: 'border-[#515157] bg-[#38383d] text-[#e2e2e7]',
  success: 'border-[#39a653] bg-[#24863b] text-white',
  warning: 'border-[#d88b13] bg-[#a96200] text-white',
  danger: 'border-[#d84b43] bg-[#a62f29] text-white',
  info: 'border-[#2488dd] bg-[#0962aa] text-white',
};

export function Badge({ children, variant = 'default', className }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-[2px] border px-2 py-0.5',
        'text-[10px] font-semibold tracking-wide',
        variants[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}
