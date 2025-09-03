import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import '../styles/globals.css'
import '../styles/african-themes.css'

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'AfroCreate Collective',
  description: 'A Web3 platform where African creators crowdfund culturally-rich content projects, mint them as revenue-sharing NFTs, and build communities around authentic African storytelling.',
  keywords: ['Web3', 'NFT', 'African', 'Creators', 'Crowdfunding', 'Cultural'],
  authors: [{ name: 'AfroCreate Collective' }],
  openGraph: {
    title: 'AfroCreate Collective',
    description: 'Empowering African creators through Web3 technology',
    type: 'website',
  },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={`${inter.className} african-pattern min-h-screen bg-gradient-to-br from-gray-900 via-wakanda-purple-900 to-black text-white`}>
        <div className="wakanda-lines min-h-screen">
          {children}
        </div>
      </body>
    </html>
  )
}