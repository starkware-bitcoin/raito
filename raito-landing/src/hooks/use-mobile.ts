"use client";

import { useState, useEffect } from "react";

export interface MobileDevice {
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  isTouchDevice: boolean;
  screenSize: "sm" | "md" | "lg" | "xl";
  orientation: "portrait" | "landscape";
  hasNotch: boolean;
  supportsHaptics: boolean;
}

export function useMobile(): MobileDevice {
  const [device, setDevice] = useState<MobileDevice>({
    isMobile: false,
    isTablet: false,
    isDesktop: true,
    isTouchDevice: false,
    screenSize: "lg",
    orientation: "landscape",
    hasNotch: false,
    supportsHaptics: false,
  });

  useEffect(() => {
    const updateDevice = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      const isTouchDevice =
        "ontouchstart" in window || navigator.maxTouchPoints > 0;

      // Screen size detection
      let screenSize: "sm" | "md" | "lg" | "xl" = "lg";
      if (width < 640) screenSize = "sm";
      else if (width < 768) screenSize = "md";
      else if (width < 1024) screenSize = "lg";
      else screenSize = "xl";

      // Device type detection
      const isMobile = width < 768;
      const isTablet = width >= 768 && width < 1024 && isTouchDevice;
      const isDesktop = width >= 1024 || !isTouchDevice;

      // Orientation
      const orientation = height > width ? "portrait" : "landscape";

      // Notch detection (iPhone X+, Android with notch)
      const hasNotch =
        "CSS" in window &&
        CSS.supports("padding", "env(safe-area-inset-top)") &&
        parseInt(
          getComputedStyle(document.documentElement).getPropertyValue(
            "env(safe-area-inset-top)"
          ) || "0"
        ) > 0;

      // Haptic feedback support
      const supportsHaptics =
        "vibrate" in navigator || "hapticActuators" in navigator;

      setDevice({
        isMobile,
        isTablet,
        isDesktop,
        isTouchDevice,
        screenSize,
        orientation,
        hasNotch,
        supportsHaptics,
      });
    };

    updateDevice();
    window.addEventListener("resize", updateDevice);
    window.addEventListener("orientationchange", updateDevice);

    return () => {
      window.removeEventListener("resize", updateDevice);
      window.removeEventListener("orientationchange", updateDevice);
    };
  }, []);

  return device;
}

// Mobile-specific utilities
export function hapticFeedback(type: "light" | "medium" | "heavy" = "light") {
  if ("vibrate" in navigator) {
    const patterns = {
      light: [10],
      medium: [20],
      heavy: [50],
    };
    navigator.vibrate(patterns[type]);
  }
}

export function preventZoom() {
  const viewport = document.querySelector('meta[name="viewport"]');
  if (viewport) {
    viewport.setAttribute(
      "content",
      "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"
    );
  }
}

export function enableZoom() {
  const viewport = document.querySelector('meta[name="viewport"]');
  if (viewport) {
    viewport.setAttribute(
      "content",
      "width=device-width, initial-scale=1.0, maximum-scale=5.0, user-scalable=yes"
    );
  }
}

// iOS-specific mobile optimizations
export function optimizeForIOS() {
  // Disable iOS zoom on double tap
  let lastTouchEnd = 0;
  document.addEventListener(
    "touchend",
    (event) => {
      const now = new Date().getTime();
      if (now - lastTouchEnd <= 300) {
        event.preventDefault();
      }
      lastTouchEnd = now;
    },
    false
  );

  // Prevent iOS scroll bounce and zoom
  document.addEventListener(
    "touchmove",
    (event) => {
      // Check if this is a pinch/zoom gesture
      if (event.touches && event.touches.length > 1) {
        event.preventDefault();
      }
    },
    { passive: false }
  );
}

// Mobile keyboard utilities
export function handleMobileKeyboard() {
  const initialHeight = window.innerHeight;

  const handleResize = () => {
    const currentHeight = window.innerHeight;
    const heightDiff = initialHeight - currentHeight;

    // Keyboard is likely open if height difference is significant
    if (heightDiff > 150) {
      document.body.classList.add("keyboard-open");
      document.documentElement.style.setProperty(
        "--keyboard-height",
        `${heightDiff}px`
      );
    } else {
      document.body.classList.remove("keyboard-open");
      document.documentElement.style.setProperty("--keyboard-height", "0px");
    }
  };

  window.addEventListener("resize", handleResize);
  return () => window.removeEventListener("resize", handleResize);
}

export default useMobile;
