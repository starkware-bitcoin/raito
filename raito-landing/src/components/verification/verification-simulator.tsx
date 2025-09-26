"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Icons } from "@/components/ui/icons"

interface VerificationSimulatorProps {
  blockHeight: number
}

type VerificationState = 'idle' | 'verifying' | 'success' | 'error'

export default function VerificationSimulator({ blockHeight }: VerificationSimulatorProps) {
  const [state, setState] = useState<VerificationState>('idle')
  
  const handleVerification = async () => {
    setState('verifying')
    
    // Simulate verification process with delay
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // For demo purposes, always succeed
    setState('success')
    
    // Reset after 3 seconds
    setTimeout(() => {
      setState('idle')
    }, 3000)
  }
  
  const getButtonContent = () => {
    switch (state) {
      case 'idle':
        return (
          <>
            <Icons.lock className="mr-2 h-4 w-4" />
            Verify Locally
          </>
        )
      case 'verifying':
        return (
          <>
            <Icons.spinner className="mr-2 h-4 w-4 animate-spin" />
            Verifying...
          </>
        )
      case 'success':
        return (
          <>
            <Icons.verified className="mr-2 h-4 w-4 animate-lock-in" />
            Verified!
          </>
        )
      case 'error':
        return (
          <>
            <Icons.unverified className="mr-2 h-4 w-4" />
            Failed
          </>
        )
    }
  }
  
  const getButtonVariant = () => {
    switch (state) {
      case 'success':
        return 'default' as const
      case 'error':
        return 'destructive' as const
      default:
        return 'outline' as const
    }
  }
  
  const getButtonClass = () => {
    switch (state) {
      case 'success':
        return 'w-full verification-glow bg-success hover:bg-success/90 text-black'
      case 'verifying':
        return 'w-full cursor-not-allowed'
      default:
        return 'w-full'
    }
  }
  
  return (
    <div className="space-y-3">
      <Button
        variant={getButtonVariant()}
        className={getButtonClass()}
        onClick={handleVerification}
        disabled={state === 'verifying'}
      >
        {getButtonContent()}
      </Button>
      
      {state === 'verifying' && (
        <div className="space-y-2 text-sm text-text-secondary">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-bitcoin rounded-full animate-pulse" />
            <span>Validating proof structure...</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-bitcoin rounded-full animate-pulse" style={{ animationDelay: '0.5s' }} />
            <span>Checking cryptographic validity...</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-bitcoin rounded-full animate-pulse" style={{ animationDelay: '1s' }} />
            <span>Verifying block #{blockHeight}...</span>
          </div>
        </div>
      )}
      
      {state === 'success' && (
        <div className="p-3 bg-success/10 border border-success/30 rounded-lg">
          <div className="flex items-center gap-2 text-success">
            <Icons.verified className="h-4 w-4" />
            <span className="font-medium">Proof verified successfully!</span>
          </div>
          <p className="text-xs text-text-secondary mt-1">
            Block #{blockHeight} is valid and part of the canonical Bitcoin chain.
          </p>
        </div>
      )}
    </div>
  )
} 