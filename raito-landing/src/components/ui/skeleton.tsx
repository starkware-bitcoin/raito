import { cn } from "@/lib/utils"

function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("skeleton rounded-md", className)}
      {...props}
    />
  )
}

function BlockCubeSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn("min-w-[280px] h-[160px] rounded-lg border border-slate-600 bg-surface overflow-hidden", className)}>
      <div className="p-4 h-full flex flex-col justify-between">
        {/* Header */}
        <div>
          <Skeleton className="h-6 w-20 mb-2" />
          <Skeleton className="h-4 w-24 mb-2" />
        </div>

        {/* Main Info */}
        <div className="space-y-2">
          <Skeleton className="h-6 w-32" />
          <Skeleton className="h-4 w-28" />
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between">
          <Skeleton className="h-3 w-16" />
          <Skeleton className="h-3 w-12" />
        </div>
      </div>
    </div>
  )
}

function StatCardSkeleton({ className }: { className?: string }) {
  return (
    <div className={cn("bg-surface rounded-xl border border-slate-800 p-6", className)}>
      <div className="flex items-center gap-2 mb-4">
        <Skeleton className="h-5 w-5 rounded" />
        <Skeleton className="h-6 w-24" />
      </div>
      <div className="space-y-3">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="flex justify-between">
            <Skeleton className="h-4 w-20" />
            <Skeleton className="h-4 w-16" />
          </div>
        ))}
      </div>
    </div>
  )
}

function TextSkeleton({ 
  lines = 3, 
  className 
}: { 
  lines?: number
  className?: string 
}) {
  return (
    <div className={cn("space-y-2", className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton 
          key={i} 
          className={cn(
            "h-4",
            i === lines - 1 ? "w-3/4" : "w-full"
          )}
        />
      ))}
    </div>
  )
}

function HashSkeleton({ className }: { className?: string }) {
  return (
    <Skeleton className={cn("h-4 w-32 font-mono", className)} />
  )
}

export { 
  Skeleton, 
  BlockCubeSkeleton, 
  StatCardSkeleton, 
  TextSkeleton,
  HashSkeleton 
} 