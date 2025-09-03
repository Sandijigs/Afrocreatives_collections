'use client'

import { motion } from 'framer-motion'
import { ArrowRight, Globe, Users, Zap, Heart, DollarSign, Shield } from 'lucide-react'

export default function HeroSection() {
  const stats = [
    { icon: Users, label: 'African Creators', value: '2,500+' },
    { icon: DollarSign, label: 'Funds Raised', value: '$1.2M+' },
    { icon: Globe, label: 'Countries', value: '54' },
    { icon: Shield, label: 'Cultural Authenticity', value: '98%' },
  ]

  const features = [
    {
      icon: Zap,
      title: 'Instant Funding',
      description: 'Get your creative projects funded in minutes, not months',
    },
    {
      icon: Heart,
      title: 'Cultural Authenticity',
      description: 'Community-verified cultural heritage validation',
    },
    {
      icon: Globe,
      title: 'Global Reach',
      description: 'Connect with supporters worldwide through Web3',
    },
  ]

  return (
    <div className="relative min-h-screen flex items-center justify-center px-4 pt-16">
      {/* Background Elements */}
      <div className="absolute inset-0 overflow-hidden">
        <motion.div
          className="absolute top-1/4 left-1/4 w-96 h-96 bg-wakanda-purple-500/10 rounded-full blur-3xl"
          animate={{ scale: [1, 1.2, 1], rotate: [0, 180, 360] }}
          transition={{ duration: 20, repeat: Infinity, ease: 'linear' }}
        />
        <motion.div
          className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-wakanda-gold-500/10 rounded-full blur-3xl"
          animate={{ scale: [1.2, 1, 1.2], rotate: [360, 180, 0] }}
          transition={{ duration: 15, repeat: Infinity, ease: 'linear' }}
        />
      </div>

      <div className="max-w-7xl mx-auto grid lg:grid-cols-2 gap-12 items-center relative z-10">
        {/* Left Column - Main Content */}
        <motion.div
          className="text-center lg:text-left"
          initial={{ opacity: 0, x: -50 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
        >
          <motion.h1
            className="text-5xl md:text-7xl font-heading font-bold mb-6 leading-tight"
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.4 }}
          >
            <span className="text-gradient">AfroCreate</span>
            <br />
            <span className="text-white">Collective</span>
          </motion.h1>

          <motion.p
            className="text-xl md:text-2xl text-gray-300 mb-8 max-w-2xl"
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.6 }}
          >
            A Web3 platform where African creators crowdfund culturally-rich content projects, 
            mint them as revenue-sharing NFTs, and build communities around authentic African storytelling.
          </motion.p>

          <motion.div
            className="flex flex-col sm:flex-row gap-4 justify-center lg:justify-start"
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.8 }}
          >
            <motion.button
              className="btn-african text-lg px-8 py-4 flex items-center justify-center gap-2"
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
            >
              Start Creating
              <ArrowRight className="w-5 h-5" />
            </motion.button>
            
            <motion.button
              className="px-8 py-4 text-lg border-2 border-wakanda-gold-500 text-wakanda-gold-500 rounded-lg hover:bg-wakanda-gold-500 hover:text-black transition-all duration-300"
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
            >
              Explore Projects
            </motion.button>
          </motion.div>
        </motion.div>

        {/* Right Column - Features Grid */}
        <motion.div
          className="space-y-8"
          initial={{ opacity: 0, x: 50 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.8, delay: 0.4 }}
        >
          <div className="grid gap-6">
            {features.map((feature, index) => (
              <motion.div
                key={feature.title}
                className="african-card p-6 tribal-hover"
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.8 + index * 0.2 }}
              >
                <div className="flex items-start gap-4">
                  <div className="p-3 bg-wakanda-gold-500/20 rounded-lg">
                    <feature.icon className="w-6 h-6 text-wakanda-gold-500" />
                  </div>
                  <div>
                    <h3 className="text-xl font-semibold text-white mb-2">
                      {feature.title}
                    </h3>
                    <p className="text-gray-400">
                      {feature.description}
                    </p>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>
      </div>

      {/* Stats Section */}
      <motion.div
        className="absolute bottom-8 left-1/2 transform -translate-x-1/2 w-full max-w-6xl px-4"
        initial={{ opacity: 0, y: 50 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, delay: 1.2 }}
      >
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
          {stats.map((stat, index) => (
            <motion.div
              key={stat.label}
              className="text-center"
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 1.4 + index * 0.1 }}
            >
              <div className="flex justify-center mb-2">
                <stat.icon className="w-8 h-8 text-wakanda-gold-500" />
              </div>
              <div className="text-2xl md:text-3xl font-bold text-gradient mb-1">
                {stat.value}
              </div>
              <div className="text-sm text-gray-400">
                {stat.label}
              </div>
            </motion.div>
          ))}
        </div>
      </motion.div>
    </div>
  )
}