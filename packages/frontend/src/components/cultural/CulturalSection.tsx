'use client'

import { motion } from 'framer-motion'
import { Shield, Award, Globe, CheckCircle, Star, Users } from 'lucide-react'
import { useState } from 'react'

export default function CulturalSection() {
  const [activeRegion, setActiveRegion] = useState('west')

  const regions = [
    {
      id: 'west',
      name: 'West Africa',
      countries: ['Nigeria', 'Ghana', 'Senegal', 'Mali', 'Burkina Faso'],
      validators: 45,
      projects: 234,
      authenticity: 96,
      description: 'Rich in musical traditions, storytelling, and vibrant artistic expressions',
      cultures: ['Yoruba', 'Akan', 'Wolof', 'Mandinka', 'Mossi'],
    },
    {
      id: 'east',
      name: 'East Africa',
      countries: ['Kenya', 'Tanzania', 'Ethiopia', 'Uganda', 'Rwanda'],
      validators: 32,
      projects: 156,
      authenticity: 94,
      description: 'Diverse cultural heritage spanning ancient civilizations to modern innovations',
      cultures: ['Maasai', 'Kikuyu', 'Amhara', 'Baganda', 'Tutsi'],
    },
    {
      id: 'south',
      name: 'Southern Africa',
      countries: ['South Africa', 'Zimbabwe', 'Botswana', 'Zambia', 'Namibia'],
      validators: 28,
      projects: 128,
      authenticity: 95,
      description: 'Ancient rock art traditions merged with contemporary creative movements',
      cultures: ['Zulu', 'Xhosa', 'Shona', 'Tswana', 'Bemba'],
    },
    {
      id: 'north',
      name: 'North Africa',
      countries: ['Egypt', 'Morocco', 'Tunisia', 'Algeria', 'Libya'],
      validators: 38,
      projects: 189,
      authenticity: 97,
      description: 'Ancient civilizations meeting Mediterranean and Middle Eastern influences',
      cultures: ['Arab', 'Berber', 'Coptic', 'Nubian', 'Tuareg'],
    },
  ]

  const validationProcess = [
    {
      step: 1,
      title: 'Community Review',
      description: 'Projects are reviewed by cultural experts from the relevant region',
      icon: Users,
    },
    {
      step: 2,
      title: 'Authenticity Check',
      description: 'Verification of cultural elements, traditions, and historical accuracy',
      icon: Shield,
    },
    {
      step: 3,
      title: 'Expert Validation',
      description: 'Final approval by certified cultural ambassadors and historians',
      icon: Award,
    },
    {
      step: 4,
      title: 'Badge Assignment',
      description: 'Authenticated projects receive cultural authenticity badges',
      icon: CheckCircle,
    },
  ]

  const currentRegion = regions.find(r => r.id === activeRegion) || regions[0]

  return (
    <div className="min-h-screen pt-24 px-4">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <motion.div
          className="text-center mb-12"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <h2 className="text-4xl md:text-6xl font-heading font-bold mb-6">
            <span className="text-gradient">Cultural Heritage</span>
          </h2>
          <p className="text-xl text-gray-300 max-w-3xl mx-auto">
            Preserving authenticity through community-driven validation and celebrating the rich diversity of African cultures
          </p>
        </motion.div>

        {/* Cultural Validation Process */}
        <motion.div
          className="mb-16"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
        >
          <h3 className="text-2xl font-heading font-bold text-center text-white mb-8">
            Our Validation Process
          </h3>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {validationProcess.map((process, index) => (
              <motion.div
                key={process.step}
                className="african-card p-6 text-center"
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.4 + index * 0.1 }}
              >
                <div className="relative mb-4">
                  <div className="w-16 h-16 bg-wakanda-gold-500/20 rounded-full flex items-center justify-center mx-auto">
                    <process.icon className="w-8 h-8 text-wakanda-gold-500" />
                  </div>
                  <div className="absolute -top-2 -right-2 w-8 h-8 bg-wakanda-purple-500 rounded-full flex items-center justify-center text-white font-bold text-sm">
                    {process.step}
                  </div>
                </div>
                <h4 className="text-lg font-semibold text-white mb-2">
                  {process.title}
                </h4>
                <p className="text-gray-400 text-sm">
                  {process.description}
                </p>
              </motion.div>
            ))}
          </div>
        </motion.div>

        {/* Regional Overview */}
        <motion.div
          className="mb-16"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.6 }}
        >
          <h3 className="text-2xl font-heading font-bold text-center text-white mb-8">
            Cultural Regions
          </h3>
          
          {/* Region Selector */}
          <div className="flex justify-center mb-8">
            <div className="flex bg-black/30 rounded-lg p-1 backdrop-blur-md overflow-x-auto">
              {regions.map((region) => (
                <motion.button
                  key={region.id}
                  onClick={() => setActiveRegion(region.id)}
                  className={`px-4 py-2 rounded-md font-medium whitespace-nowrap transition-all duration-300 ${
                    activeRegion === region.id
                      ? 'bg-wakanda-gold-500 text-black'
                      : 'text-gray-300 hover:text-white'
                  }`}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                >
                  {region.name}
                </motion.button>
              ))}
            </div>
          </div>

          {/* Selected Region Details */}
          <motion.div
            key={activeRegion}
            className="african-card p-8"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
          >
            <div className="grid lg:grid-cols-2 gap-8">
              <div>
                <h4 className="text-2xl font-bold text-white mb-4">
                  {currentRegion.name}
                </h4>
                <p className="text-gray-300 mb-6">
                  {currentRegion.description}
                </p>

                {/* Countries */}
                <div className="mb-6">
                  <h5 className="text-lg font-semibold text-wakanda-gold-400 mb-3">
                    Countries
                  </h5>
                  <div className="flex flex-wrap gap-2">
                    {currentRegion.countries.map((country) => (
                      <span
                        key={country}
                        className="px-3 py-1 bg-wakanda-purple-500/20 text-wakanda-purple-300 rounded-full text-sm"
                      >
                        {country}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Cultural Groups */}
                <div>
                  <h5 className="text-lg font-semibold text-wakanda-gold-400 mb-3">
                    Major Cultural Groups
                  </h5>
                  <div className="flex flex-wrap gap-2">
                    {currentRegion.cultures.map((culture) => (
                      <span
                        key={culture}
                        className="px-3 py-1 bg-wakanda-gold-500/20 text-wakanda-gold-300 rounded-full text-sm"
                      >
                        {culture}
                      </span>
                    ))}
                  </div>
                </div>
              </div>

              {/* Stats */}
              <div className="grid grid-cols-2 gap-6">
                <div className="text-center">
                  <div className="text-3xl font-bold text-gradient mb-2">
                    {currentRegion.validators}
                  </div>
                  <div className="text-gray-400 text-sm">Cultural Validators</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-gradient mb-2">
                    {currentRegion.projects}
                  </div>
                  <div className="text-gray-400 text-sm">Validated Projects</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-gradient mb-2">
                    {currentRegion.authenticity}%
                  </div>
                  <div className="text-gray-400 text-sm">Authenticity Rate</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-gradient mb-2">
                    4.9
                  </div>
                  <div className="text-gray-400 text-sm">Validator Rating</div>
                </div>
              </div>
            </div>
          </motion.div>
        </motion.div>

        {/* Authenticity Badge Showcase */}
        <motion.div
          className="text-center"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.8 }}
        >
          <h3 className="text-2xl font-heading font-bold text-white mb-6">
            Authenticity Matters
          </h3>
          <p className="text-gray-300 mb-8 max-w-2xl mx-auto">
            Every project displaying our cultural authenticity badge has been verified by community experts, 
            ensuring genuine representation of African heritage and traditions.
          </p>
          
          <div className="flex justify-center">
            <motion.div
              className="authenticity-verified text-lg px-8 py-4 flex items-center gap-3"
              whileHover={{ scale: 1.05 }}
              animate={{ boxShadow: ['0 0 20px rgba(34, 139, 34, 0.3)', '0 0 30px rgba(34, 139, 34, 0.5)', '0 0 20px rgba(34, 139, 34, 0.3)'] }}
              transition={{ duration: 2, repeat: Infinity }}
            >
              <CheckCircle className="w-6 h-6" />
              Culturally Authentic
              <Star className="w-6 h-6" />
            </motion.div>
          </div>
        </motion.div>
      </div>
    </div>
  )
}