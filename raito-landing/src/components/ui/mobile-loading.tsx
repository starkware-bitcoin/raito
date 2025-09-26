"use client"

import { useState, useEffect } from "react"
import { Icons } from "@/components/ui/icons"

interface MobileLoadingProps {
  isLoading: boolean
  loadingText?: string
  showPullHint?: boolean
  children: React.ReactNode
}

export function MobileLoading({ 
  isLoading, 
  loadingText = "Loading...", 
  showPullHint = false,
  children 
}: MobileLoadingProps) {
  const [showHint, setShowHint] = useState(showPullHint)

  useEffect(() => {
    if (showPullHint) {
      const timer = setTimeout(() => setShowHint(false), 3000)
      return () => clearTimeout(timer)
    }
  }, [showPullHint])

  if (isLoading) {
    return (
      <div className="mobile-loading-container">
        {/* Pull to refresh hint */}
        {showHint && (
          <div className="fixed top-20 left-1/2 transform -translate-x-1/2 z-40 mobile-slide-up">
            <div className="flex items-center gap-2 px-4 py-2 bg-bitcoin text-black rounded-full shadow-lg">
              <Icons.activity className="h-4 w-4 animate-spin" />
              <span className="text-sm font-medium">Pull down to refresh</span>
            </div>
          </div>
        )}

        {/* Mobile loading screen */}
        <div className="min-h-screen bg-background mobile-p-safe flex flex-col">
          {/* Loading header */}
          <div className="p-4 border-b border-slate-700">
            <div className="skeleton-premium h-8 w-48 rounded-lg mx-auto"></div>
          </div>

          {/* Loading content */}
          <div className="flex-1 p-4 space-y-4">
            <div className="text-center space-y-4">
              <div className="relative">
                <div className="w-16 h-16 mx-auto bg-bitcoin/20 rounded-2xl flex items-center justify-center">
                  <Icons.activity className="h-8 w-8 text-bitcoin animate-spin" />
                </div>
                <div className="absolute inset-0 bg-bitcoin/10 rounded-2xl animate-ping"></div>
              </div>
              
              <div className="space-y-2">
                <div className="text-lg font-semibold text-text-primary">{loadingText}</div>
                <div className="text-sm text-text-secondary">Verifying with STARK proofs...</div>
              </div>

              {/* Loading dots */}
              <div className="loading-dots-premium flex justify-center gap-2 mt-4">
                <span className="dot w-2 h-2"></span>
                <span className="dot w-2 h-2"></span>
                <span className="dot w-2 h-2"></span>
              </div>
            </div>

            {/* Skeleton cards */}
            <div className="space-y-4 mt-8">
              {[...Array(3)].map((_, i) => (
                <div key={i} className="skeleton-card p-4 bg-surface rounded-xl border border-slate-700">
                  <div className="flex items-center space-x-3">
                    <div className="skeleton-premium w-12 h-12 rounded-lg"></div>
                    <div className="flex-1 space-y-2">
                      <div className="skeleton-premium h-4 w-3/4 rounded"></div>
                      <div className="skeleton-premium h-3 w-1/2 rounded"></div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    )
  }

  return <>{children}</>
}

// Mobile skeleton components
export function MobileBlockSkeleton() {
  return (
    <div className="space-y-4 mobile-p-safe">
      {[...Array(5)].map((_, i) => (
        <div key={i} className="p-4 bg-surface rounded-xl border border-slate-700 space-y-3">
          <div className="flex items-center justify-between">
            <div className="skeleton-premium h-6 w-24 rounded"></div>
            <div className="skeleton-premium h-5 w-16 rounded"></div>
          </div>
          <div className="skeleton-premium h-4 w-full rounded"></div>
          <div className="flex justify-between">
            <div className="skeleton-premium h-4 w-20 rounded"></div>
            <div className="skeleton-premium h-4 w-20 rounded"></div>
          </div>
        </div>
      ))}
    </div>
  )
}

export function MobileFormSkeleton() {
  return (
    <div className="space-y-6 mobile-p-safe p-4">
      <div className="space-y-4">
        <div className="skeleton-premium h-8 w-48 rounded"></div>
        <div className="skeleton-premium h-12 w-full rounded-lg"></div>
        <div className="skeleton-premium h-10 w-full rounded-lg"></div>
      </div>
      
      <div className="space-y-3">
        <div className="skeleton-premium h-6 w-32 rounded"></div>
        <div className="skeleton-premium h-20 w-full rounded-lg"></div>
      </div>
    </div>
  )
}

// Touch feedback component
export function TouchFeedback({ children, onTouch }: { 
  children: React.ReactNode
  onTouch?: () => void 
}) {
  const [isPressed, setIsPressed] = useState(false)

  const handleTouchStart = () => {
    setIsPressed(true)
    onTouch?.()
  }

  const handleTouchEnd = () => {
    setTimeout(() => setIsPressed(false), 150)
  }

  return (
    <div
      className={`transition-transform duration-150 ${isPressed ? 'scale-95' : 'scale-100'}`}
      onTouchStart={handleTouchStart}
      onTouchEnd={handleTouchEnd}
      onMouseDown={handleTouchStart}
      onMouseUp={handleTouchEnd}
    >
      {children}
    </div>
  )
} 