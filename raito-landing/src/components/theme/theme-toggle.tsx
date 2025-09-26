"use client"

import { useEffect, useState } from "react"
import { useTheme } from './theme-provider'
import { Button } from '@/components/ui/button'
import { Icons } from '@/components/ui/icons'

export function ThemeToggle() {
  const { theme, toggleTheme } = useTheme()
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
  }, [])

  // Prevent hydration mismatch by not rendering theme-specific content until mounted
  if (!mounted) {
    return (
      <Button
        variant="ghost"
        size="sm"
        className="relative h-10 w-10 p-0 theme-transition hover:bg-surface-alt/80 group overflow-hidden border border-transparent hover:border-bitcoin/20 rounded-xl"
        aria-label="Loading theme toggle"
        disabled
      >
        <div className="absolute inset-0 flex items-center justify-center">
          <Icons.sun className="h-5 w-5 text-bitcoin/50" />
        </div>
      </Button>
    )
  }

  return (
    <div className="relative">
      <Button
        variant="ghost"
        size="sm"
        onClick={toggleTheme}
        className="relative h-10 w-10 p-0 theme-transition hover:bg-surface-alt/80 group overflow-hidden border border-transparent hover:border-bitcoin/20 rounded-xl"
        aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} mode`}
      >
        {/* Background glow effect */}
        <div className={`
          absolute inset-0 rounded-xl bg-gradient-to-br from-bitcoin/10 to-transparent 
          opacity-0 transition-all duration-500 ease-out
          group-hover:opacity-100 group-hover:scale-110
        `} />
        
        {/* Rotating background for theme transition */}
        <div className={`
          absolute inset-0 rounded-xl transition-all duration-700 ease-out
          ${theme === 'dark' 
            ? 'bg-gradient-to-br from-slate-800/50 to-slate-900/50 rotate-0' 
            : 'bg-gradient-to-br from-orange-100/50 to-yellow-100/50 rotate-180'
          }
        `} />
        
        {/* Sun icon for light mode */}
        <div className={`
          absolute inset-0 flex items-center justify-center
          transition-all duration-500 ease-out
          ${theme === 'dark' 
            ? 'scale-0 rotate-180 opacity-0' 
            : 'scale-100 rotate-0 opacity-100'
          }
        `}>
          <Icons.sun className="h-5 w-5 text-bitcoin drop-shadow-lg" />
          <div className={`
            absolute inset-0 rounded-full
            ${theme === 'light' ? 'animate-ping bg-bitcoin/20' : ''}
          `} />
        </div>
        
        {/* Moon icon for dark mode */}
        <div className={`
          absolute inset-0 flex items-center justify-center
          transition-all duration-500 ease-out
          ${theme === 'dark' 
            ? 'scale-100 rotate-0 opacity-100' 
            : 'scale-0 -rotate-180 opacity-0'
          }
        `}>
          <Icons.moon className="h-5 w-5 text-bitcoin drop-shadow-lg" />
          <div className={`
            absolute inset-0 rounded-full
            ${theme === 'dark' ? 'animate-pulse bg-bitcoin/10' : ''}
          `} />
        </div>
        
        {/* Orbital ring animation */}
        <div className={`
          absolute inset-0 rounded-xl border-2 border-transparent
          transition-all duration-300 ease-out
          group-hover:border-bitcoin/30 group-hover:shadow-lg
          ${theme === 'dark' 
            ? 'shadow-[0_0_20px_rgba(247,147,26,0.1)]' 
            : 'shadow-[0_0_20px_rgba(247,147,26,0.2)]'
          }
        `} />
        
        {/* Click ripple effect */}
        <div className="ripple-premium absolute inset-0 rounded-xl" />
        
        <span className="sr-only">Toggle theme</span>
      </Button>
      
      {/* Theme status indicator */}
      <div className={`
        absolute -bottom-1 left-1/2 transform -translate-x-1/2
        w-2 h-2 rounded-full transition-all duration-500 ease-out
        ${theme === 'dark' 
          ? 'bg-slate-400 shadow-[0_0_8px_rgba(148,163,184,0.5)]' 
          : 'bg-bitcoin shadow-[0_0_8px_rgba(247,147,26,0.7)]'
        }
      `} />
    </div>
  )
} 