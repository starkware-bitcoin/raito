"use client"

import { useState } from "react"

interface HashFlickerProps {
  hash: string
  className?: string
  truncate?: boolean
  copyable?: boolean
}

export function HashFlicker({ 
  hash, 
  className = "", 
  truncate = false,
  copyable = false 
}: HashFlickerProps) {
  const [isAnimating, setIsAnimating] = useState(false)
  const [showCopyFeedback, setShowCopyFeedback] = useState(false)

  const displayHash = truncate && hash.length > 20 
    ? `${hash.slice(0, 8)}...${hash.slice(-8)}`
    : hash

  const handleCopy = async () => {
    if (copyable) {
      try {
        await navigator.clipboard.writeText(hash)
        setShowCopyFeedback(true)
        setTimeout(() => setShowCopyFeedback(false), 2000)
      } catch (err) {
        console.error('Failed to copy hash:', err)
      }
    }
  }

  const handleMouseEnter = () => {
    setIsAnimating(true)
    setTimeout(() => setIsAnimating(false), 1200) // Match animation duration
  }

  return (
    <div className="relative inline-block group">
      <span
        className={`
          hash-flicker font-mono text-sm relative cursor-pointer
          transition-all duration-300 ease-out
          hover:text-bitcoin hover:drop-shadow-[0_0_8px_rgba(247,147,26,0.4)]
          ${isAnimating ? 'text-bitcoin' : ''}
          ${className}
        `}
        onMouseEnter={handleMouseEnter}
        onClick={handleCopy}
        title={copyable ? `Click to copy: ${hash}` : hash}
      >
        {/* Background glow effect */}
        <span className={`
          absolute inset-0 rounded px-1 transition-all duration-300
          ${isAnimating 
            ? 'bg-bitcoin/10 shadow-[0_0_20px_rgba(247,147,26,0.2)]' 
            : 'group-hover:bg-bitcoin/5'
          }
        `} />
        
        {/* Hash characters with scramble effect */}
        <span className="hash-chars relative z-10 break-all leading-relaxed">
          {displayHash.split('').map((char, index) => (
            <span
              key={index}
              className={`
                inline-block transition-all duration-75 ease-out
                ${isAnimating ? 'animate-pulse' : ''}
              `}
              style={{
                '--char-index': index,
                animationDelay: `${index * 0.03}s`
              } as React.CSSProperties}
            >
              {char}
            </span>
          ))}
        </span>

        {/* Animated underline */}
        <span className={`
          absolute bottom-0 left-0 h-0.5 bg-bitcoin transition-all duration-300 ease-out
          ${isAnimating || (copyable && 'group-hover:w-full') ? 'w-full' : 'w-0'}
        `} />
        
        {/* Copy indicator */}
        {copyable && (
          <span className={`
            absolute -top-8 left-1/2 transform -translate-x-1/2
            px-2 py-1 bg-bitcoin text-black text-xs rounded font-medium
            transition-all duration-300 ease-out pointer-events-none
            ${showCopyFeedback 
              ? 'opacity-100 translate-y-0 scale-100' 
              : 'opacity-0 translate-y-2 scale-95'
            }
          `}>
            Copied!
            <div className="absolute top-full left-1/2 transform -translate-x-1/2 border-4 border-transparent border-t-bitcoin" />
          </span>
        )}
      </span>

      {/* Floating particles effect during animation */}
      {isAnimating && (
        <div className="absolute inset-0 pointer-events-none">
          {[...Array(6)].map((_, i) => (
            <div
              key={i}
              className={`
                absolute w-1 h-1 bg-bitcoin rounded-full opacity-60
                animate-ping
              `}
              style={{
                left: `${20 + i * 15}%`,
                top: `${10 + (i % 2) * 20}%`,
                animationDelay: `${i * 0.2}s`,
                animationDuration: '1s'
              }}
            />
          ))}
        </div>
      )}
    </div>
  )
} 