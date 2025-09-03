/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    domains: ['localhost', 'ipfs.io'],
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
}

module.exports = nextConfig