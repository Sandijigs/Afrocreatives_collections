'use client'

import { motion } from 'framer-motion'
import { useState } from 'react'
import { Menu, X, Zap } from 'lucide-react'
import { cn } from '@/utils/cn'

interface HeaderProps {
  activeSection: string
  setActiveSection: (section: string) => void
}

export default function Header({ activeSection, setActiveSection }: HeaderProps) {
  const [isMenuOpen, setIsMenuOpen] = useState(false)

  const navigation = [
    { id: 'hero', label: 'Home' },
    { id: 'creators', label: 'Creators' },
    { id: 'projects', label: 'Projects' },
    { id: 'cultural', label: 'Cultural Heritage' },
    { id: 'ens', label: 'Identity & ENS' },
  ]

  return (
    <motion.header
      className="fixed top-0 left-0 right-0 z-50 bg-black/20 backdrop-blur-md border-b border-wakanda-gold-500/20"
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      transition={{ duration: 0.6, ease: 'easeOut' }}
    >
      <nav className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <motion.div
            className="flex items-center space-x-2 cursor-pointer"
            onClick={() => setActiveSection('hero')}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            <div className="relative">
              <Zap className="w-8 h-8 text-wakanda-gold-500" />
              <motion.div
                className="absolute inset-0 bg-wakanda-gold-500 rounded-full opacity-20"
                animate={{ scale: [1, 1.2, 1] }}
                transition={{ duration: 2, repeat: Infinity }}
              />
            </div>
            <span className="text-xl font-heading font-bold text-gradient">
              AfroCreate
            </span>
          </motion.div>

          {/* Desktop Navigation */}
          <div className="hidden md:block">
            <div className="ml-10 flex items-baseline space-x-4">
              {navigation.map((item) => (
                <motion.button
                  key={item.id}
                  onClick={() => setActiveSection(item.id)}
                  className={cn(
                    'px-3 py-2 rounded-md text-sm font-medium transition-all duration-300',
                    activeSection === item.id
                      ? 'text-wakanda-gold-500 bg-wakanda-gold-500/10'
                      : 'text-gray-300 hover:text-wakanda-gold-400 hover:bg-wakanda-gold-400/5'
                  )}
                  whileHover={{ y: -2 }}
                  whileTap={{ y: 0 }}
                >
                  {item.label}
                </motion.button>
              ))}
            </div>
          </div>

          {/* Connect Wallet Button */}
          <motion.button
            className="hidden md:block btn-african text-sm"
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            Connect Wallet
          </motion.button>

          {/* Mobile menu button */}
          <div className="md:hidden">
            <motion.button
              onClick={() => setIsMenuOpen(!isMenuOpen)}
              className="text-gray-400 hover:text-wakanda-gold-400"
              whileTap={{ scale: 0.95 }}
            >
              {isMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
            </motion.button>
          </div>
        </div>

        {/* Mobile Navigation */}
        <motion.div
          className={cn(
            'md:hidden overflow-hidden',
            isMenuOpen ? 'max-h-96 pb-4' : 'max-h-0'
          )}
          initial={false}
          animate={{ 
            maxHeight: isMenuOpen ? 384 : 0,
            opacity: isMenuOpen ? 1 : 0
          }}
          transition={{ duration: 0.3 }}
        >
          <div className="px-2 pt-2 pb-3 space-y-1 sm:px-3">
            {navigation.map((item) => (
              <motion.button
                key={item.id}
                onClick={() => {
                  setActiveSection(item.id)
                  setIsMenuOpen(false)
                }}
                className={cn(
                  'block px-3 py-2 rounded-md text-base font-medium w-full text-left transition-all duration-300',
                  activeSection === item.id
                    ? 'text-wakanda-gold-500 bg-wakanda-gold-500/10'
                    : 'text-gray-300 hover:text-wakanda-gold-400 hover:bg-wakanda-gold-400/5'
                )}
                whileHover={{ x: 4 }}
              >
                {item.label}
              </motion.button>
            ))}
            <motion.button
              className="w-full mt-4 btn-african text-sm"
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
            >
              Connect Wallet
            </motion.button>
          </div>
        </motion.div>
      </nav>
    </motion.header>
  )
}