import { forwardRef } from 'react';
import { cn } from '@/lib/utils';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
}

const base = cn(
  'inline-flex items-center justify-center gap-1.5 rounded-[2px] font-semibold',
  'border cursor-default transition-[background-color,border-color,box-shadow,transform] duration-150',
  'focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-[#0a84ff]/25',
  'disabled:opacity-45 disabled:pointer-events-none active:scale-[0.985]',
);

const variants: Record<NonNullable<ButtonProps['variant']>, string> = {
  primary: cn(
    'border-[#3095ff] bg-[#0a84ff] text-white',
    'shadow-[inset_0_1px_0_rgba(255,255,255,0.2),0_1px_2px_rgba(0,0,0,0.18)]',
    'hover:border-[#58a8ff] hover:bg-[#2692ff]',
  ),
  secondary: cn(
    'border-[#45454a] bg-[#2c2c30] text-[#f5f5f7]',
    'shadow-[inset_0_1px_0_rgba(255,255,255,0.04),0_1px_2px_rgba(0,0,0,0.12)]',
    'hover:border-[#5a5a60] hover:bg-[#36363a]',
  ),
  ghost: cn(
    'border-[#353539] bg-[#232326] text-[#a1a1a6]',
    'hover:border-[#4a4a4f] hover:bg-[#2d2d31] hover:text-white',
  ),
  danger: cn(
    'border-[#ff6259] bg-[#d93730] text-white',
    'shadow-[inset_0_1px_0_rgba(255,255,255,0.18),0_1px_2px_rgba(0,0,0,0.18)]',
    'hover:bg-[#ff5e55]',
  ),
};

const sizes: Record<NonNullable<ButtonProps['size']>, string> = {
  sm: 'h-8 px-3 text-[12px]',
  md: 'h-9 px-3.5 text-[13px]',
  lg: 'h-10 px-5 text-[14px]',
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
