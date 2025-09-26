"use client"

import Link from "next/link"
import Image from "next/image"
import { usePathname } from "next/navigation"
import { useState, useEffect } from "react"
import { cn } from "@/lib/utils"
import { ThemeToggle } from "@/components/theme/theme-toggle"
import { Icons } from "@/components/ui/icons"

const navigation: Array<{ name: string; href: string; icon?: React.ComponentType<{ className?: string }>; description?: string }> = []

export default function Navigation() {
  const pathname = usePathname()
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)

  // Close mobile menu when route changes
  useEffect(() => {
    setIsMobileMenuOpen(false)
  }, [pathname])

  // Prevent body scroll when mobile menu is open
  useEffect(() => {
    if (isMobileMenuOpen) {
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = ''
    }
    
    return () => {
      document.body.style.overflow = ''
    }
  }, [isMobileMenuOpen])

  const toggleMobileMenu = () => {
    setIsMobileMenuOpen(!isMobileMenuOpen)
  }

  return (
    <>
      <nav className="sticky top-0 z-50 border-b border-slate-800 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 theme-transition mobile-p-safe">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 justify-between items-center">
            {/* Logo */}
            <div className="flex items-center">
              <Link href="/" className="flex items-center group" onClick={() => setIsMobileMenuOpen(false)}>
                <div className="flex items-center space-x-3">
                  <div className="relative h-8 w-8 group-hover:scale-110 transition-transform duration-300">
                    <Image
                      src="/raito-logo.png"
                      alt="Raito Logo"
                      fill
                      sizes="(max-width: 768px) 2rem, 2rem"
                      className="object-contain"
                      priority
                    />
                  </div>
                  <div className="flex flex-col">
                    <span className="font-bold text-xl text-text-primary group-hover:text-bitcoin transition-colors">Raito</span>
                  </div>
                </div>
              </Link>
            </div>

            {/* Desktop Navigation */}
            <div className="hidden md:ml-8 md:flex md:space-x-8">
              {navigation.map((item) => {
                const Icon = item.icon
                return (
                  <Link
                    key={item.name}
                    href={item.href}
                    className={cn(
                      "inline-flex items-center px-1 pt-1 text-sm font-medium transition-all duration-300 hover:scale-105 group",
                      (pathname === "/" && item.href.startsWith("#")) || pathname === item.href
                        ? "text-bitcoin border-b-2 border-bitcoin"
                        : "text-text-secondary hover:text-text-primary hover:border-slate-600 border-b-2 border-transparent"
                    )}
                  >
                    {Icon && <Icon className="h-4 w-4 mr-2 group-hover:scale-110 transition-transform" />}
                    {item.name}
                  </Link>
                )
              })}
            </div>

            {/* Desktop Actions */}
            <div className="hidden md:flex md:items-center space-x-3">
              <ThemeToggle />
              <Link
                href="https://github.com/starkware-bitcoin/raito"
                target="_blank"
                rel="noopener noreferrer"
                className="text-text-secondary hover:text-bitcoin transition-all duration-300 hover:scale-110 p-2 rounded-md hover:bg-surface-alt/50"
              >
                <span className="sr-only">GitHub</span>
                <svg className="h-6 w-6" fill="currentColor" viewBox="0 0 24 24">
                  <path
                    fillRule="evenodd"
                    d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                    clipRule="evenodd"
                  />
                </svg>
              </Link>
            </div>

            {/* Mobile Menu Button & Theme Toggle */}
            <div className="flex items-center space-x-2 md:hidden">
              <ThemeToggle />
              <button
                onClick={toggleMobileMenu}
                className={cn(
                  "mobile-icon-button p-2 rounded-xl text-text-secondary transition-all duration-300",
                  "hover:text-bitcoin hover:bg-surface-alt/50 focus:outline-none focus:ring-2 focus:ring-bitcoin/30",
                  isMobileMenuOpen ? "text-bitcoin bg-surface-alt" : ""
                )}
                aria-label="Toggle mobile menu"
                aria-expanded={isMobileMenuOpen}
              >
                <div className="relative w-6 h-6">
                  <span
                    className={cn(
                      "absolute left-0 top-1 block h-0.5 w-6 bg-current transition-all duration-300 ease-out",
                      isMobileMenuOpen ? "rotate-45 translate-y-1.5" : ""
                    )}
                  />
                  <span
                    className={cn(
                      "absolute left-0 top-2.5 block h-0.5 w-6 bg-current transition-all duration-300 ease-out",
                      isMobileMenuOpen ? "opacity-0" : ""
                    )}
                  />
                  <span
                    className={cn(
                      "absolute left-0 top-4 block h-0.5 w-6 bg-current transition-all duration-300 ease-out",
                      isMobileMenuOpen ? "-rotate-45 -translate-y-1.5" : ""
                    )}
                  />
                </div>
              </button>
            </div>
          </div>
        </div>

        {/* Mobile Menu Overlay */}
        {isMobileMenuOpen && (
          <div 
            className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm md:hidden"
            onClick={() => setIsMobileMenuOpen(false)}
            aria-hidden="true"
          />
        )}

        {/* Mobile Menu Panel */}
        <div
          className={cn(
            "fixed right-0 top-0 z-50 h-full w-80 max-w-sm transform bg-background/98 backdrop-blur-xl shadow-2xl transition-all duration-300 ease-out md:hidden mobile-p-safe",
            isMobileMenuOpen ? "translate-x-0" : "translate-x-full"
          )}
        >
          <div className="flex h-full flex-col">
            {/* Mobile Menu Header */}
            <div className="flex items-center justify-between p-4 border-b border-slate-700">
              <div className="flex items-center space-x-3">
                <div className="relative h-8 w-8">
                  <Image
                    src="/raito-logo.png"
                    alt="Raito Logo"
                    fill
                    sizes="(max-width: 768px) 2rem, 2rem"
                    className="object-contain"
                    priority
                  />
                </div>
                <span className="font-bold text-xl text-text-primary">Raito</span>
              </div>
              <button
                onClick={() => setIsMobileMenuOpen(false)}
                className="mobile-icon-button p-2 rounded-xl text-text-secondary hover:text-danger hover:bg-danger/10 transition-colors"
                aria-label="Close mobile menu"
              >
                <Icons.unverified className="h-6 w-6" />
              </button>
            </div>

            {/* Mobile Navigation Links */}
            <div className="flex-1 overflow-y-auto mobile-scroll">
              <div className="p-4 space-y-2">
                {navigation.map((item, index) => {
                  const Icon = item.icon
                  const isActive = (pathname === "/" && item.href.startsWith("#")) || pathname === item.href
                  
                  return (
                    <Link
                      key={item.name}
                      href={item.href}
                      className={cn(
                        "group flex items-center p-4 rounded-xl transition-all duration-300 mobile-hover touch-target",
                        isActive
                          ? "bg-bitcoin/10 text-bitcoin border border-bitcoin/30"
                          : "text-text-secondary hover:text-text-primary hover:bg-surface-alt border border-transparent"
                      )}
                      style={{ animationDelay: `${index * 0.1}s` }}
                    >
                      <div className={cn(
                        "flex items-center justify-center w-10 h-10 rounded-lg mr-4 transition-all duration-300",
                        isActive ? "bg-bitcoin/20" : "bg-surface-alt group-hover:bg-surface"
                      )}>
                        {Icon && <Icon className={cn(
                          "h-5 w-5 transition-transform duration-300",
                          isActive ? "text-bitcoin" : "text-text-secondary group-hover:text-text-primary group-hover:scale-110"
                        )} />}
                      </div>
                      <div>
                        <div className={cn(
                          "font-semibold text-base",
                          isActive ? "text-bitcoin" : ""
                        )}>
                          {item.name}
                        </div>
                        <div className="text-sm text-text-secondary mt-0.5">
                          {item.description}
                        </div>
                      </div>
                    </Link>
                  )
                })}
              </div>

              {/* Mobile Menu Footer */}
              <div className="p-4 border-t border-slate-700 mt-4">
                <Link
                  href="https://github.com/starkware-bitcoin/raito"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center p-4 rounded-xl text-text-secondary hover:text-bitcoin hover:bg-surface-alt transition-all duration-300 mobile-hover touch-target group"
                >
                  <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-surface-alt group-hover:bg-surface mr-4">
                    <svg className="h-5 w-5 group-hover:scale-110 transition-transform duration-300" fill="currentColor" viewBox="0 0 24 24">
                      <path
                        fillRule="evenodd"
                        d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </div>
                  <div>
                    <div className="font-semibold text-base">View Source</div>
                    <div className="text-sm text-text-secondary mt-0.5">GitHub Repository</div>
                  </div>
                  <Icons.externalLink className="h-4 w-4 ml-auto opacity-50 group-hover:opacity-100 transition-opacity" />
                </Link>

                {/* App info */}
                <div className="mt-4 p-3 bg-surface/50 rounded-lg border border-slate-700">
                  <div className="text-xs text-text-secondary text-center space-y-1">
                    <div>Raito Bitcoin STARK Portal</div>
                    <div className="text-bitcoin">Don&apos;t trust, verify</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </nav>
    </>
  )
} 