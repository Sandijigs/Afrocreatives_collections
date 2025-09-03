'use client'

import { motion } from 'framer-motion'
import { Zap, Twitter, Github, MessageCircle, Mail, Globe, Heart } from 'lucide-react'

export default function Footer() {
  const links = {
    platform: [
      { name: 'For Creators', href: '#' },
      { name: 'For Backers', href: '#' },
      { name: 'Cultural Validation', href: '#' },
      { name: 'ENS Domains', href: '#' },
    ],
    community: [
      { name: 'Discord', href: '#' },
      { name: 'Twitter', href: '#' },
      { name: 'Medium', href: '#' },
      { name: 'GitHub', href: '#' },
    ],
    resources: [
      { name: 'Documentation', href: '#' },
      { name: 'API Reference', href: '#' },
      { name: 'Cultural Guidelines', href: '#' },
      { name: 'Help Center', href: '#' },
    ],
    legal: [
      { name: 'Privacy Policy', href: '#' },
      { name: 'Terms of Service', href: '#' },
      { name: 'Cultural Ethics', href: '#' },
      { name: 'Cookie Policy', href: '#' },
    ],
  }

  const socialLinks = [
    { icon: Twitter, href: '#', label: 'Twitter' },
    { icon: Github, href: '#', label: 'GitHub' },
    { icon: MessageCircle, href: '#', label: 'Discord' },
    { icon: Mail, href: '#', label: 'Email' },
  ]

  const africanCountries = [
    'Nigeria', 'Kenya', 'Ghana', 'South Africa', 'Egypt', 'Morocco', 
    'Ethiopia', 'Tanzania', 'Uganda', 'Senegal', 'Mali', 'Rwanda'
  ]

  return (
    <footer className="relative mt-20 bg-black/40 backdrop-blur-md border-t border-wakanda-gold-500/20">
      {/* Main Footer Content */}
      <div className="max-w-7xl mx-auto px-4 py-16">
        <div className="grid lg:grid-cols-5 gap-8">
          {/* Brand Section */}
          <div className="lg:col-span-2">
            <motion.div
              className="flex items-center space-x-2 mb-6"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6 }}
              viewport={{ once: true }}
            >
              <div className="relative">
                <Zap className="w-8 h-8 text-wakanda-gold-500" />
                <motion.div
                  className="absolute inset-0 bg-wakanda-gold-500 rounded-full opacity-20"
                  animate={{ scale: [1, 1.2, 1] }}
                  transition={{ duration: 2, repeat: Infinity }}
                />
              </div>
              <span className="text-2xl font-heading font-bold text-gradient">
                AfroCreate
              </span>
            </motion.div>

            <motion.p
              className="text-gray-300 mb-6 max-w-md"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.1 }}
              viewport={{ once: true }}
            >
              Empowering African creators to fund culturally-rich projects, build communities, 
              and share authentic stories with the world through Web3 technology.
            </motion.p>

            {/* Social Links */}
            <motion.div
              className="flex space-x-4"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.2 }}
              viewport={{ once: true }}
            >
              {socialLinks.map((social, index) => (
                <motion.a
                  key={social.label}
                  href={social.href}
                  className="p-3 bg-wakanda-gold-500/10 rounded-lg text-wakanda-gold-400 hover:text-wakanda-gold-300 hover:bg-wakanda-gold-500/20 transition-colors"
                  whileHover={{ scale: 1.1, y: -2 }}
                  whileTap={{ scale: 0.95 }}
                  aria-label={social.label}
                >
                  <social.icon className="w-5 h-5" />
                </motion.a>
              ))}
            </motion.div>
          </div>

          {/* Links Sections */}
          {Object.entries(links).map(([category, items], categoryIndex) => (
            <motion.div
              key={category}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.1 * (categoryIndex + 1) }}
              viewport={{ once: true }}
            >
              <h3 className="text-lg font-semibold text-white mb-4 capitalize">
                {category}
              </h3>
              <ul className="space-y-3">
                {items.map((link, index) => (
                  <li key={index}>
                    <motion.a
                      href={link.href}
                      className="text-gray-400 hover:text-wakanda-gold-400 transition-colors"
                      whileHover={{ x: 4 }}
                    >
                      {link.name}
                    </motion.a>
                  </li>
                ))}
              </ul>
            </motion.div>
          ))}
        </div>

        {/* African Countries Ticker */}
        <motion.div
          className="mt-12 pt-8 border-t border-gray-700"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          transition={{ duration: 0.8, delay: 0.4 }}
          viewport={{ once: true }}
        >
          <div className="text-center mb-4">
            <h4 className="text-lg font-semibold text-wakanda-gold-400 mb-2">
              Supporting Creators Across Africa
            </h4>
            <div className="flex items-center justify-center gap-2 text-gray-400">
              <Globe className="w-4 h-4" />
              <span className="text-sm">Active in 54 African countries</span>
            </div>
          </div>
          
          <div className="overflow-hidden">
            <motion.div
              className="flex space-x-8 whitespace-nowrap"
              animate={{ x: ['0%', '-50%'] }}
              transition={{ duration: 30, repeat: Infinity, ease: 'linear' }}
            >
              {[...africanCountries, ...africanCountries].map((country, index) => (
                <span
                  key={index}
                  className="text-gray-500 hover:text-wakanda-gold-400 transition-colors cursor-default"
                >
                  {country}
                </span>
              ))}
            </motion.div>
          </div>
        </motion.div>
      </div>

      {/* Bottom Bar */}
      <motion.div
        className="border-t border-gray-800 py-6"
        initial={{ opacity: 0 }}
        whileInView={{ opacity: 1 }}
        transition={{ duration: 0.6, delay: 0.6 }}
        viewport={{ once: true }}
      >
        <div className="max-w-7xl mx-auto px-4">
          <div className="flex flex-col md:flex-row justify-between items-center gap-4">
            <div className="text-gray-400 text-sm">
              © 2024 AfroCreate Collective. All rights reserved.
            </div>
            
            <div className="flex items-center gap-2 text-gray-400 text-sm">
              <span>Built with</span>
              <Heart className="w-4 h-4 text-red-400" />
              <span>for African creators</span>
            </div>
            
            <div className="text-gray-400 text-sm">
              Powered by Web3 • Cultural Heritage First
            </div>
          </div>
        </div>
      </motion.div>

      {/* Background Pattern */}
      <div className="absolute inset-0 pointer-events-none opacity-5">
        <div className="kente-pattern w-full h-full" />
      </div>
    </footer>
  )
}