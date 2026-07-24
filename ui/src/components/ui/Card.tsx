import { cn } from '@/lib/utils';

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  hover?: boolean;
}

export function Card({ className, hover, children, ...props }: CardProps) {
  return (
    <div
      className={cn(
        'bg-[linear-gradient(180deg,#3e3e3e_0%,#333_100%)]',
        'border-2 border-[#555] border-t-[#5a5a5a]',
        'shadow-[inset_0_1px_0_rgba(255,255,255,0.04),0_2px_3px_rgba(0,0,0,0.3)]',
        hover && 'hover:bg-[linear-gradient(180deg,#484848_0%,#383838_100%)] hover:border-[#666] cursor-default',
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}

export function CardHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        'px-4 py-2.5 border-b-2 border-[#2a2a2a]',
        'bg-[linear-gradient(180deg,#383838_0%,#2d2d2d_100%)]',
        'font-bold text-[11px] uppercase tracking-wider text-[#c0c0b0] text-shadow',
        className,
      )}
      {...props}
    />
  );
}

export function CardContent({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('px-4 py-3', className)} {...props} />;
}
