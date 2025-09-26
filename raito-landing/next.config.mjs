/** @type {import('next').NextConfig} */
const nextConfig = {
  // Enable experimental features
  experimental: {
    optimizePackageImports: ['@vercel/analytics', '@vercel/speed-insights'],
  },

  serverExternalPackages: ['@starkware-bitcoin/spv-verify'],

  // Turbopack configuration removed - using webpack fallbacks instead

  // Webpack configuration for handling Node.js modules in browser
  webpack: (config, { isServer, webpack }) => {
    config.experiments = {
      ...(config.experiments ?? {}),
      asyncWebAssembly: true,
    };

    config.module.rules.push({
      test: /\.wasm$/,
      type: 'webassembly/async',
    });

    config.output = {
      ...config.output,
      environment: {
        ...(config.output?.environment ?? {}),
        asyncFunction: true,
      },
    };

    if (!isServer) {
      config.plugins.push(
        new webpack.ProvidePlugin({
          process: 'process/browser',
        })
      );

      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        crypto: false,
        stream: false,
        util: false,
        buffer: false,
      };
    }

    return config;
  },

  // Production optimizations
  compress: true,
  
  // Image optimization (optimized for Vercel)
  images: {
    formats: ['image/webp', 'image/avif'],
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'raito.wtf',
        port: '',
        pathname: '/**',
      },
    ],
    deviceSizes: [640, 750, 828, 1080, 1200, 1920, 2048, 3840],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
  },

  // Headers for security and SEO
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          // Security headers
          {
            key: 'X-Frame-Options',
            value: 'DENY',
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff',
          },
          {
            key: 'Referrer-Policy',
            value: 'origin-when-cross-origin',
          },
          {
            key: 'X-DNS-Prefetch-Control',
            value: 'on',
          },
          {
            key: 'X-XSS-Protection',
            value: '1; mode=block',
          },
          {
            key: 'X-Permitted-Cross-Domain-Policies',
            value: 'none',
          },
        ],
      },
      {
        source: '/favicon.ico',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
      {
        source: '/favicon.svg',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
      {
        source: '/(favicon-.*|apple-touch-icon|android-chrome-.*)\\.png',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
      {
        source: '/bitcoin-logo\\.(svg|png)',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
      {
        source: '/site.webmanifest',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=86400',
          },
          {
            key: 'Content-Type',
            value: 'application/manifest+json',
          },
        ],
      },
      {
        source: '/_next/static/(.*)',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
    ];
  },

  // PoweredByHeader
  poweredByHeader: false,

  // React strict mode
  reactStrictMode: true,

  // Eslint config during build
  eslint: {
    ignoreDuringBuilds: false,
  },

  // TypeScript config during build  
  typescript: {
    ignoreBuildErrors: false,
  },

  // Redirects for SEO (if needed)
  async redirects() {
    return [
      // Example redirect from old paths
      // {
      //   source: '/old-path',
      //   destination: '/new-path',
      //   permanent: true,
      // },
    ];
  },

  // Rewrites for clean URLs
  async rewrites() {
    return [
      // Example rewrite for clean URLs
      // {
      //   source: '/api/:path*',
      //   destination: '/api/:path*',
      // },
    ];
  },
};

export default nextConfig; 