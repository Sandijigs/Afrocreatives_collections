'use client'

import { useState } from 'react'
import { motion } from 'framer-motion'
import Header from '@/components/ui/Header'
import HeroSection from '@/components/ui/HeroSection'
import CreatorShowcase from '@/components/creators/CreatorShowcase'
import ProjectsSection from '@/components/projects/ProjectsSection'
import CulturalSection from '@/components/cultural/CulturalSection'
import ENSSection from '@/components/ens/ENSSection'
import Footer from '@/components/ui/Footer'

export default function HomePage() {
  const [activeSection, setActiveSection] = useState('hero')

  const sections = [
    { id: 'hero', title: 'Home', component: HeroSection },
    { id: 'creators', title: 'Creators', component: CreatorShowcase },
    { id: 'projects', title: 'Projects', component: ProjectsSection },
    { id: 'cultural', title: 'Cultural Heritage', component: CulturalSection },
    { id: 'ens', title: 'Identity & ENS', component: ENSSection },
  ]

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-wakanda-purple-900 to-black">
      <Header activeSection={activeSection} setActiveSection={setActiveSection} />
      
      <main className="relative">
        {sections.map(({ id, component: Component }) => (
          <motion.section
            key={id}
            id={id}
            className={`min-h-screen relative ${
              activeSection === id ? 'block' : 'hidden'
            }`}
            initial={{ opacity: 0, y: 50 }}
            animate={{ 
              opacity: activeSection === id ? 1 : 0,
              y: activeSection === id ? 0 : 50 
            }}
            transition={{ duration: 0.6, ease: 'easeOut' }}
          >
            <Component />
          </motion.section>
        ))}
      </main>

      <Footer />

      {/* Floating African Pattern Overlay */}
      <div className="fixed inset-0 pointer-events-none overflow-hidden">
        <div className="absolute top-20 left-10 w-32 h-32 opacity-10">
          <motion.div
            className="w-full h-full bg-gradient-to-br from-african-gold to-african-copper rounded-full"
            animate={{ rotate: 360 }}
            transition={{ duration: 20, repeat: Infinity, ease: 'linear' }}
          />
        </div>
        
        <div className="absolute top-1/3 right-20 w-24 h-24 opacity-10">
          <motion.div
            className="w-full h-full bg-gradient-to-br from-wakanda-purple-500 to-african-royal"
            animate={{ rotate: -360 }}
            transition={{ duration: 15, repeat: Infinity, ease: 'linear' }}
          />
        </div>
        
        <div className="absolute bottom-20 left-1/4 w-40 h-40 opacity-5">
          <motion.div
            className="w-full h-full border-2 border-african-gold rounded-full"
            animate={{ scale: [1, 1.2, 1] }}
            transition={{ duration: 4, repeat: Infinity, ease: 'easeInOut' }}
          />
        </div>
      </div>
    </div>
  )
}