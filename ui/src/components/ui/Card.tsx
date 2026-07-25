import { cn } from '@/lib/utils';

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  hover?: boolean;
}

export function Card({ className, hover, children, ...props }: CardProps) {
  return (
    <div
      className={cn(
        'overflow-hidden rounded-[3px] border border-[#38383c] bg-[#202023]',
        'shadow-[0_12px_34px_rgba(0,0,0,0.14)]',
        hover && 'cursor-default transition duration-200 hover:-translate-y-0.5 hover:border-[#505055] hover:bg-[#27272a] hover:shadow-[0_16px_40px_rgba(0,0,0,0.22)]',
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
        'border-b border-[#38383c] bg-[#262629] px-5 py-3.5',
        'text-[12px] font-semibold text-[#f5f5f7]',
        className,
      )}
      {...props}
    />
  );
}

export function CardContent({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('px-5 py-4', className)} {...props} />;
}
