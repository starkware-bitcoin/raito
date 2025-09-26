"use client"

import { useState, useRef } from "react"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Icons } from "@/components/ui/icons"

interface MobileHashInputProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
  className?: string
  disabled?: boolean
  isValid?: boolean
  isInvalid?: boolean
  onPaste?: (value: string) => void
}

export function MobileHashInput({
  value,
  onChange,
  placeholder,
  className = "",
  disabled = false,
  isValid = false,
  isInvalid = false,
  onPaste
}: MobileHashInputProps) {
  const [isFocused, setIsFocused] = useState(false)
  const [showCopyFeedback, setShowCopyFeedback] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  const formatHash = (input: string): string => {
    // Remove non-hex characters and convert to lowercase
    return input.replace(/[^a-fA-F0-9]/g, '').toLowerCase()
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const formatted = formatHash(e.target.value)
    onChange(formatted)
  }

  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText()
      const formatted = formatHash(text)
      onChange(formatted)
      onPaste?.(formatted)
      
      // Show feedback
      setShowCopyFeedback(true)
      setTimeout(() => setShowCopyFeedback(false), 2000)
    } catch {
      console.log("Clipboard access denied")
    }
  }

  const handleCopy = async () => {
    if (value) {
      try {
        await navigator.clipboard.writeText(value)
        setShowCopyFeedback(true)
        setTimeout(() => setShowCopyFeedback(false), 2000)
      } catch {
        console.log("Copy failed")
      }
    }
  }

  const handleClear = () => {
    onChange("")
    inputRef.current?.focus()
  }

  return (
    <div className={`relative ${className}`}>
      {/* Mobile-optimized input */}
      <div className="relative">
        <Input
          ref={inputRef}
          type="text"
          value={value}
          onChange={handleInputChange}
          placeholder={placeholder}
          disabled={disabled}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
          className={`
            mobile-hash-input pr-24
            ${isValid ? 'border-success focus-visible:ring-success' : ''}
            ${isInvalid ? 'border-danger focus-visible:ring-danger' : ''}
            ${isFocused ? 'ring-2 ring-bitcoin/30' : ''}
          `}
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck="false"
        />

        {/* Action buttons container */}
        <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
          {value && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={handleClear}
              className="mobile-icon-button h-8 w-8 p-1 text-text-secondary hover:text-danger"
            >
              <Icons.unverified className="h-4 w-4" />
            </Button>
          )}
          
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={value ? handleCopy : handlePaste}
            className="mobile-icon-button h-8 w-8 p-1 text-text-secondary hover:text-bitcoin"
          >
            {value ? (
              <Icons.verified className="h-4 w-4" />
            ) : (
              <Icons.activity className="h-4 w-4" />
            )}
          </Button>
        </div>
      </div>

      {/* Mobile-friendly hash display */}
      {value && value.length > 20 && !isFocused && (
        <div className="mt-2 p-3 bg-surface-alt rounded-lg border border-slate-600 mobile-only">
          <div className="text-xs text-text-secondary mb-1">Full Hash:</div>
          <div className="font-mono text-sm text-text-primary break-all leading-relaxed">
            {value}
          </div>
        </div>
      )}

      {/* Character counter */}
      <div className="flex items-center justify-between mt-2 text-xs">
        <div className={`
          transition-colors duration-200
          ${value.length === 64 ? 'text-success' : 
            value.length > 64 ? 'text-danger' : 
            'text-text-secondary'}
        `}>
          {value.length}/64 characters
        </div>
        
        {showCopyFeedback && (
          <div className="flex items-center gap-1 text-success animate-in fade-in duration-200">
            <Icons.verified className="h-3 w-3" />
            <span>{value ? 'Copied!' : 'Pasted!'}</span>
          </div>
        )}
      </div>

      {/* Mobile formatting hints */}
      {isFocused && (
        <div className="mt-2 p-2 bg-surface rounded border border-slate-700 mobile-only">
          <div className="text-xs text-text-secondary space-y-1">
            <div>💡 Paste from clipboard or type hex characters</div>
            <div>🔄 Auto-formats: removes spaces, converts to lowercase</div>
            <div>✅ Valid length: exactly 64 characters</div>
          </div>
        </div>
      )}
    </div>
  )
} 