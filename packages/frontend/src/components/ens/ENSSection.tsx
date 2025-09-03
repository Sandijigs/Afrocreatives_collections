'use client'

import { motion } from 'framer-motion'
import { useState } from 'react'
import { User, Globe, Shield, Zap, Search, CheckCircle, ExternalLink } from 'lucide-react'

export default function ENSSection() {
  const [searchDomain, setSearchDomain] = useState('')
  const [selectedFeature, setSelectedFeature] = useState('identity')

  const features = [
    {
      id: 'identity',
      title: 'African Identity',
      icon: User,
      description: 'Create your unique African creator identity with .afro domains',
      benefits: [
        'Memorable creator identity (kemi.afro)',
        'Cultural authenticity verification',
        'Cross-platform recognition',
        'Simplified wallet addresses',
      ],
    },
    {
      id: 'discovery',
      title: 'Global Discovery',
      icon: Globe,
      description: 'Make your African heritage discoverable worldwide',
      benefits: [
        'SEO-optimized cultural profiles',
        'International audience reach',
        'Cultural heritage showcase',
        'Multi-language support',
      ],
    },
    {
      id: 'security',
      title: 'Secure Identity',
      icon: Shield,
      description: 'Decentralized identity with ownership control',
      benefits: [
        'Own your digital identity',
        'Prevent identity theft',
        'Blockchain-secured profiles',
        'Verifiable credentials',
      ],
    },
    {
      id: 'integration',
      title: 'Web3 Integration',
      icon: Zap,
      description: 'Seamless integration with Web3 ecosystem',
      benefits: [
        'Connect to DeFi protocols',
        'NFT marketplace integration',
        'DAO participation rights',
        'Simplified transactions',
      ],
    },
  ]

  const exampleProfiles = [
    {
      domain: 'kemi.afro',
      name: 'Kemi Adebayo',
      specialty: 'Afrobeats Producer',
      location: 'Lagos, Nigeria',
      verified: true,
      badges: ['Music Producer', 'Cultural Ambassador', 'Top Creator'],
    },
    {
      domain: 'jelani.afro',
      name: 'Jelani Mwangi',
      specialty: 'Digital Artist',
      location: 'Nairobi, Kenya',
      verified: true,
      badges: ['NFT Artist', 'Innovation Award', 'Maasai Heritage'],
    },
    {
      domain: 'amara.afro',
      name: 'Amara Okonkwo',
      specialty: 'Film Director',
      location: 'Accra, Ghana',
      verified: true,
      badges: ['Documentary', 'Cultural Preservation', 'Storyteller'],
    },
  ]

  const currentFeature = features.find(f => f.id === selectedFeature) || features[0]

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
            <span className="text-gradient">African Identity</span>
          </h2>
          <p className="text-xl text-gray-300 max-w-3xl mx-auto">
            Own your digital identity with .afro domains. Build your reputation, showcase your culture, 
            and connect with the global African creative community.
          </p>
        </motion.div>

        {/* Domain Search */}
        <motion.div
          className="african-card p-8 mb-12 text-center"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
        >
          <h3 className="text-2xl font-bold text-white mb-6">
            Claim Your .afro Identity
          </h3>
          
          <div className="max-w-md mx-auto mb-6">
            <div className="relative">
              <input
                type="text"
                placeholder="Enter your desired name"
                value={searchDomain}
                onChange={(e) => setSearchDomain(e.target.value)}
                className="w-full px-4 py-4 bg-black/30 border border-gray-600 rounded-l-lg text-white placeholder-gray-400 focus:border-wakanda-gold-500 focus:outline-none"
              />
              <div className="absolute right-0 top-0 bottom-0 bg-wakanda-gold-500 px-6 rounded-r-lg flex items-center text-black font-semibold">
                .afro
              </div>
            </div>
          </div>

          <motion.button
            className="btn-african text-lg px-8 py-3 flex items-center gap-2 mx-auto"
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            <Search className="w-5 h-5" />
            Check Availability
          </motion.button>

          <p className="text-gray-400 text-sm mt-4">
            Join 2,500+ African creators with verified .afro identities
          </p>
        </motion.div>

        {/* Feature Showcase */}
        <motion.div
          className="mb-12"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4 }}
        >
          {/* Feature Tabs */}
          <div className="flex justify-center mb-8 overflow-x-auto">
            <div className="flex bg-black/30 rounded-lg p-1 backdrop-blur-md">
              {features.map((feature) => (
                <motion.button
                  key={feature.id}
                  onClick={() => setSelectedFeature(feature.id)}
                  className={`px-4 py-3 rounded-md font-medium whitespace-nowrap transition-all duration-300 flex items-center gap-2 ${
                    selectedFeature === feature.id
                      ? 'bg-wakanda-gold-500 text-black'
                      : 'text-gray-300 hover:text-white'
                  }`}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                >
                  <currentFeature.icon className="w-4 h-4" />
                  {feature.title}
                </motion.button>
              ))}
            </div>
          </div>

          {/* Feature Content */}
          <motion.div
            key={selectedFeature}
            className="african-card p-8"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
          >
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <div>
                <div className="flex items-center gap-3 mb-4">
                  <div className="p-3 bg-wakanda-gold-500/20 rounded-lg">
                    <currentFeature.icon className="w-8 h-8 text-wakanda-gold-500" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">
                    {currentFeature.title}
                  </h3>
                </div>
                
                <p className="text-gray-300 mb-6 text-lg">
                  {currentFeature.description}
                </p>

                <div className="space-y-3">
                  {currentFeature.benefits.map((benefit, index) => (
                    <motion.div
                      key={index}
                      className="flex items-center gap-3"
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      transition={{ duration: 0.4, delay: index * 0.1 }}
                    >
                      <CheckCircle className="w-5 h-5 text-green-400 flex-shrink-0" />
                      <span className="text-gray-300">{benefit}</span>
                    </motion.div>
                  ))}
                </div>
              </div>

              <div>
                <h4 className="text-lg font-semibold text-white mb-4">
                  Featured Profiles
                </h4>
                <div className="space-y-4">
                  {exampleProfiles.map((profile, index) => (
                    <motion.div
                      key={profile.domain}
                      className="p-4 bg-black/20 rounded-lg border border-gray-700 hover:border-wakanda-gold-500/30 transition-colors"
                      initial={{ opacity: 0, y: 20 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ duration: 0.4, delay: 0.2 + index * 0.1 }}
                    >
                      <div className="flex items-start justify-between mb-2">
                        <div>
                          <div className="flex items-center gap-2 mb-1">
                            <span className="text-wakanda-gold-400 font-semibold">
                              {profile.domain}
                            </span>
                            {profile.verified && (
                              <CheckCircle className="w-4 h-4 text-green-400" />
                            )}
                          </div>
                          <h5 className="text-white font-medium">{profile.name}</h5>
                          <p className="text-sm text-gray-400">{profile.specialty} • {profile.location}</p>
                        </div>
                        <ExternalLink className="w-4 h-4 text-gray-400 hover:text-wakanda-gold-400 cursor-pointer" />
                      </div>
                      
                      <div className="flex flex-wrap gap-1 mt-3">
                        {profile.badges.slice(0, 2).map((badge) => (
                          <span
                            key={badge}
                            className="px-2 py-1 bg-wakanda-purple-500/20 text-wakanda-purple-300 text-xs rounded"
                          >
                            {badge}
                          </span>
                        ))}
                      </div>
                    </motion.div>
                  ))}
                </div>
              </div>
            </div>
          </motion.div>
        </motion.div>

        {/* Call to Action */}
        <motion.div
          className="text-center"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.8 }}
        >
          <h3 className="text-2xl font-heading font-bold text-white mb-4">
            Ready to Claim Your Identity?
          </h3>
          <p className="text-gray-300 mb-6 max-w-2xl mx-auto">
            Join the movement of African creators building their digital legacy. 
            Own your identity, showcase your culture, and connect with supporters worldwide.
          </p>
          <motion.button
            className="btn-african text-lg px-8 py-4"
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            Get Your .afro Domain
          </motion.button>
        </motion.div>
      </div>
    </div>
  )
}