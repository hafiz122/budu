import { cn } from '@/lib/utils';

interface BadgeProps {
  children: React.ReactNode;
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
  className?: string;
}

const variants = {
  default: 'bg-[linear-gradient(180deg,#555_0%,#3a3a3a_100%)] border-[#444] text-[#c0c0b0]',
  success: 'bg-[linear-gradient(180deg,#6a9f3c_0%,#4a7f1c_100%)] border-[#3a6f0c] text-white',
  warning: 'bg-[linear-gradient(180deg,#d4ac30_0%,#b48c10_100%)] border-[#947000] text-white',
  danger: 'bg-[linear-gradient(180deg,#cc4444_0%,#aa2222_100%)] border-[#881111] text-white',
  info: 'bg-[linear-gradient(180deg,#4488cc_0%,#2266aa_100%)] border-[#114488] text-white',
};

export function Badge({ children, variant = 'default', className }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide',
        'border shadow-[inset_0_1px_0_rgba(255,255,255,0.1)] text-shadow',
        variants[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}
