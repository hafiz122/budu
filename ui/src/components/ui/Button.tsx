import { forwardRef } from 'react';
import { cn } from '@/lib/utils';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
}

const base = cn(
  'inline-flex items-center justify-center gap-1.5 font-bold uppercase tracking-wide',
  'border-2 cursor-default transition-none',
  'focus-visible:outline-none disabled:opacity-60 disabled:pointer-events-none',
  'active:translate-y-px',
);

const variants: Record<NonNullable<ButtonProps['variant']>, string> = {
  primary: cn(
    'bg-[linear-gradient(180deg,#8db834_0%,#6a8c1e_50%,#5a7c0e_100%)]',
    'text-white text-shadow',
    'border-[#4a6c0e] border-t-[#a0cc40]',
    'shadow-[inset_0_1px_0_rgba(255,255,255,0.2),0_2px_3px_rgba(0,0,0,0.4)]',
    'hover:bg-[linear-gradient(180deg,#9dc844_0%,#7a9c2e_50%,#6a8c1e_100%)]',
    'active:shadow-[inset_0_2px_4px_rgba(0,0,0,0.4)]',
  ),
  secondary: cn(
    'bg-[linear-gradient(180deg,#5a5a5a_0%,#444_50%,#383838_100%)]',
    'text-[#e0e0d0] text-shadow',
    'border-[#333] border-t-[#6a6a6a]',
    'shadow-[inset_0_1px_0_rgba(255,255,255,0.06),0_2px_3px_rgba(0,0,0,0.3)]',
    'hover:bg-[linear-gradient(180deg,#666_0%,#4a4a4a_50%,#3e3e3e_100%)]',
  ),
  ghost: cn(
    'border-transparent bg-transparent text-[#a0a090]',
    'hover:bg-[#444] hover:text-[#e0e0d0]',
  ),
  danger: cn(
    'bg-[linear-gradient(180deg,#cc4444_0%,#aa2222_50%,#991111_100%)]',
    'text-white text-shadow',
    'border-[#771111] border-t-[#dd5555]',
    'shadow-[inset_0_1px_0_rgba(255,255,255,0.15),0_2px_3px_rgba(0,0,0,0.4)]',
  ),
};

const sizes: Record<NonNullable<ButtonProps['size']>, string> = {
  sm: 'px-2.5 py-1 text-[10px]',
  md: 'px-3 py-1.5 text-[11px]',
  lg: 'px-4 py-2 text-[12px]',
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'primary', size = 'md', ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(base, variants[variant], sizes[size], className)}
        {...props}
      />
    );
  },
);

Button.displayName = 'Button';
