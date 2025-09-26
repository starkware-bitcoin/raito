"use client"

import React, { createContext, useContext, useEffect, useState } from 'react'

type Theme = 'dark' | 'light'

type ThemeProviderProps = {
  children: React.ReactNode
  defaultTheme?: Theme
  storageKey?: string
}

type ThemeProviderState = {
  theme: Theme
  setTheme: (theme: Theme) => void
  toggleTheme: () => void
}

const initialState: ThemeProviderState = {
  theme: 'dark',
  setTheme: () => null,
  toggleTheme: () => null,
}

const ThemeProviderContext = createContext<ThemeProviderState>(initialState)

export function ThemeProvider({
  children,
  defaultTheme = 'dark',
  storageKey = 'raito-theme',
  ...props
}: ThemeProviderProps) {
  // Always start with defaultTheme to prevent hydration mismatch
  const [theme, setTheme] = useState<Theme>(defaultTheme)
  const [mounted, setMounted] = useState(false)

  // After hydration, check localStorage and update theme if needed
  useEffect(() => {
    setMounted(true)
    
    const storedTheme = localStorage.getItem(storageKey) as Theme
    if (storedTheme && (storedTheme === 'dark' || storedTheme === 'light')) {
      setTheme(storedTheme)
    }
  }, [storageKey])

  // Apply theme to DOM
  useEffect(() => {
    if (!mounted) return // Don't apply until after hydration
    
    const root = window.document.documentElement
    
    root.classList.remove('light', 'dark')
    root.classList.add(theme)
  }, [theme, mounted])

  const value = {
    theme,
    setTheme: (theme: Theme) => {
      localStorage.setItem(storageKey, theme)
      setTheme(theme)
    },
    toggleTheme: () => {
      const newTheme = theme === 'dark' ? 'light' : 'dark'
      localStorage.setItem(storageKey, newTheme)
      setTheme(newTheme)
    },
  }

  // Prevent hydration mismatch by not rendering theme-dependent content until mounted
  if (!mounted) {
    return (
      <ThemeProviderContext.Provider {...props} value={{ ...value, theme: defaultTheme }}>
        {children}
      </ThemeProviderContext.Provider>
    )
  }

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  )
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext)

  if (context === undefined)
    throw new Error('useTheme must be used within a ThemeProvider')

  return context
} 