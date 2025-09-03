'use client'

import { motion } from 'framer-motion'
import { useState } from 'react'
import { Star, MapPin, Users, DollarSign, Play, Heart } from 'lucide-react'
import CreatorCard from './CreatorCard'
import CreatorProfile from './CreatorProfile'
import CreatorRegistration from './CreatorRegistration'

export default function CreatorShowcase() {
  const [activeTab, setActiveTab] = useState('showcase')
  const [selectedCreator, setSelectedCreator] = useState<typeof creators[0] | null>(null)

  const creators = [
    {
      id: 1,
      name: 'Kemi Adebayo',
      avatar: '/avatars/kemi.jpg',
      location: 'Lagos, Nigeria',
      specialty: 'Afrobeats Producer',
      followers: 15400,
      verified: true,
      rating: 4.9,
      currentProject: 'Rhythms of Lagos',
      raised: '$12,500',
      goal: '$15,000',
      description: 'Creating authentic Afrobeats that tell the story of modern Nigeria',
      achievements: ['Top Producer 2023', 'Cultural Ambassador', 'Rising Star'],
    },
    {
      id: 2,
      name: 'Amara Okonkwo',
      avatar: '/avatars/amara.jpg',
      location: 'Accra, Ghana',
      specialty: 'Film Director',
      followers: 8200,
      verified: true,
      rating: 4.8,
      currentProject: 'Stories from the Volta',
      raised: '$8,750',
      goal: '$20,000',
      description: 'Documenting Ghanaian culture through contemporary storytelling',
      achievements: ['Best Documentary', 'Cultural Preservation Award'],
    },
    {
      id: 3,
      name: 'Jelani Mwangi',
      avatar: '/avatars/jelani.jpg',
      location: 'Nairobi, Kenya',
      specialty: 'Digital Artist',
      followers: 22100,
      verified: true,
      rating: 4.9,
      currentProject: 'Maasai Futures',
      raised: '$18,200',
      goal: '$25,000',
      description: 'Blending traditional Maasai art with futuristic digital concepts',
      achievements: ['NFT Artist of the Year', 'Innovation Award', 'Cultural Bridge Builder'],
    },
  ]

  const tabs = [
    { id: 'showcase', label: 'Featured Creators' },
    { id: 'profile', label: 'Creator Profiles' },
    { id: 'register', label: 'Join as Creator' },
  ]

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
            <span className="text-gradient">African Creators</span>
          </h2>
          <p className="text-xl text-gray-300 max-w-3xl mx-auto">
            Discover talented African creators building the future of cultural storytelling through Web3 technology
          </p>
        </motion.div>

        {/* Tabs */}
        <motion.div
          className="flex justify-center mb-12"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
        >
          <div className="flex bg-black/30 rounded-lg p-1 backdrop-blur-md">
            {tabs.map((tab) => (
              <motion.button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`px-6 py-3 rounded-md font-medium transition-all duration-300 ${
                  activeTab === tab.id
                    ? 'bg-wakanda-gold-500 text-black'
                    : 'text-gray-300 hover:text-white'
                }`}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
              >
                {tab.label}
              </motion.button>
            ))}
          </div>
        </motion.div>

        {/* Content */}
        <motion.div
          key={activeTab}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
        >
          {activeTab === 'showcase' && (
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
              {creators.map((creator, index) => (
                <motion.div
                  key={creator.id}
                  initial={{ opacity: 0, y: 30 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.6, delay: index * 0.2 }}
                >
                  <CreatorCard 
                    creator={creator} 
                    onClick={() => setSelectedCreator(creator)}
                  />
                </motion.div>
              ))}
            </div>
          )}

          {activeTab === 'profile' && (
            <CreatorProfile 
              creator={selectedCreator || creators[0]}
              onBack={() => setSelectedCreator(null)}
            />
          )}

          {activeTab === 'register' && <CreatorRegistration />}
        </motion.div>

        {/* Creator Stats */}
        <motion.div
          className="mt-16 grid grid-cols-2 md:grid-cols-4 gap-8"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4 }}
        >
          {[
            { icon: Users, label: 'Active Creators', value: '2,500+' },
            { icon: DollarSign, label: 'Total Earned', value: '$1.2M+' },
            { icon: Play, label: 'Projects Launched', value: '890+' },
            { icon: Heart, label: 'Community Support', value: '98%' },
          ].map((stat, index) => (
            <motion.div
              key={stat.label}
              className="text-center african-card p-6"
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.6 + index * 0.1 }}
            >
              <div className="flex justify-center mb-4">
                <stat.icon className="w-8 h-8 text-wakanda-gold-500" />
              </div>
              <div className="text-2xl font-bold text-gradient mb-2">
                {stat.value}
              </div>
              <div className="text-gray-400">
                {stat.label}
              </div>
            </motion.div>
          ))}
        </motion.div>
      </div>
    </div>
  )
}