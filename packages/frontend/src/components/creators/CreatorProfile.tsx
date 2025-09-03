'use client'

import { motion } from 'framer-motion'
import { ArrowLeft, MapPin, Star, Users, DollarSign, CheckCircle, Calendar, Award, ExternalLink } from 'lucide-react'

interface Creator {
  id: number
  name: string
  avatar: string
  location: string
  specialty: string
  followers: number
  verified: boolean
  rating: number
  currentProject: string
  raised: string
  goal: string
  description: string
  achievements: string[]
}

interface CreatorProfileProps {
  creator: Creator
  onBack: () => void
}

export default function CreatorProfile({ creator, onBack }: CreatorProfileProps) {
  const projects = [
    {
      id: 1,
      title: creator.currentProject,
      status: 'active',
      raised: creator.raised,
      goal: creator.goal,
      backers: 124,
      timeLeft: '12 days',
      description: 'An immersive journey through the vibrant music scene of Lagos, showcasing authentic Afrobeats culture.',
    },
    {
      id: 2,
      title: 'Ancestral Echoes',
      status: 'completed',
      raised: '$8,500',
      goal: '$7,000',
      backers: 89,
      timeLeft: 'Completed',
      description: 'A tribute to traditional Nigerian music with modern production techniques.',
    },
  ]

  return (
    <div className="max-w-4xl mx-auto">
      {/* Back Button */}
      <motion.button
        onClick={onBack}
        className="flex items-center gap-2 text-wakanda-gold-400 hover:text-wakanda-gold-300 mb-8"
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        whileHover={{ x: -5 }}
      >
        <ArrowLeft className="w-4 h-4" />
        Back to Creators
      </motion.button>

      {/* Profile Header */}
      <motion.div
        className="african-card p-8 mb-8"
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <div className="flex flex-col md:flex-row gap-6">
          <div className="flex-shrink-0">
            <div className="relative">
              <div className="w-32 h-32 bg-gradient-to-br from-wakanda-gold-500 to-wakanda-purple-500 rounded-full p-1">
                <div className="w-full h-full bg-gray-800 rounded-full flex items-center justify-center text-4xl font-bold text-wakanda-gold-500">
                  {creator.name.split(' ').map(n => n[0]).join('')}
                </div>
              </div>
              {creator.verified && (
                <div className="absolute -top-2 -right-2 bg-wakanda-gold-500 rounded-full p-2">
                  <CheckCircle className="w-6 h-6 text-black" />
                </div>
              )}
            </div>
          </div>

          <div className="flex-1">
            <div className="flex items-start justify-between mb-4">
              <div>
                <h1 className="text-3xl font-heading font-bold text-white mb-2">
                  {creator.name}
                </h1>
                <div className="flex items-center gap-4 text-gray-300 mb-3">
                  <div className="flex items-center gap-1">
                    <MapPin className="w-4 h-4" />
                    {creator.location}
                  </div>
                  <div className="flex items-center gap-1">
                    <Star className="w-4 h-4 text-wakanda-gold-500 fill-current" />
                    {creator.rating} rating
                  </div>
                </div>
              </div>
              
              <motion.button
                className="btn-african"
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
              >
                Follow Creator
              </motion.button>
            </div>

            <div className="cultural-badge mb-4">
              {creator.specialty}
            </div>

            <p className="text-gray-300 mb-6">
              {creator.description}. With years of experience in the African music industry, 
              they bring authentic cultural perspectives to every project while embracing modern production techniques.
            </p>

            {/* Stats */}
            <div className="grid grid-cols-3 gap-6">
              <div className="text-center">
                <div className="text-2xl font-bold text-gradient">{creator.followers.toLocaleString()}</div>
                <div className="text-sm text-gray-400">Followers</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-gradient">23</div>
                <div className="text-sm text-gray-400">Projects</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-gradient">$45.2K</div>
                <div className="text-sm text-gray-400">Total Raised</div>
              </div>
            </div>
          </div>
        </div>
      </motion.div>

      {/* Achievements */}
      <motion.div
        className="african-card p-6 mb-8"
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.2 }}
      >
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <Award className="w-5 h-5 text-wakanda-gold-500" />
          Achievements
        </h2>
        <div className="flex flex-wrap gap-3">
          {creator.achievements.map((achievement, index) => (
            <motion.div
              key={index}
              className="px-4 py-2 bg-wakanda-purple-500/20 text-wakanda-purple-300 rounded-full border border-wakanda-purple-500/30"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.4, delay: 0.4 + index * 0.1 }}
            >
              {achievement}
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Projects */}
      <motion.div
        className="african-card p-6"
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.4 }}
      >
        <h2 className="text-xl font-semibold text-white mb-6">Projects</h2>
        <div className="space-y-6">
          {projects.map((project, index) => (
            <motion.div
              key={project.id}
              className="p-4 bg-black/20 rounded-lg border border-gray-700 hover:border-wakanda-gold-500/30 transition-colors"
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.4, delay: 0.6 + index * 0.2 }}
            >
              <div className="flex justify-between items-start mb-3">
                <div>
                  <h3 className="text-lg font-semibold text-white mb-1">{project.title}</h3>
                  <p className="text-gray-400 text-sm">{project.description}</p>
                </div>
                <span className={`px-3 py-1 rounded-full text-xs font-medium ${
                  project.status === 'active' 
                    ? 'bg-green-500/20 text-green-400' 
                    : 'bg-blue-500/20 text-blue-400'
                }`}>
                  {project.status}
                </span>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                <div>
                  <div className="text-gray-400">Raised</div>
                  <div className="text-wakanda-gold-400 font-medium">{project.raised}</div>
                </div>
                <div>
                  <div className="text-gray-400">Goal</div>
                  <div className="text-white">{project.goal}</div>
                </div>
                <div>
                  <div className="text-gray-400">Backers</div>
                  <div className="text-white">{project.backers}</div>
                </div>
                <div>
                  <div className="text-gray-400">Status</div>
                  <div className="text-white">{project.timeLeft}</div>
                </div>
              </div>

              {project.status === 'active' && (
                <div className="mt-4">
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div 
                      className="bg-gradient-to-r from-wakanda-gold-500 to-wakanda-purple-500 h-2 rounded-full"
                      style={{ width: `${(parseInt(project.raised.replace(/[^0-9]/g, '')) / parseInt(project.goal.replace(/[^0-9]/g, ''))) * 100}%` }}
                    />
                  </div>
                </div>
              )}
            </motion.div>
          ))}
        </div>
      </motion.div>
    </div>
  )
}