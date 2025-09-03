'use client'

import { motion } from 'framer-motion'
import { MapPin, Star, Users, DollarSign, CheckCircle, Heart, ExternalLink } from 'lucide-react'

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

interface CreatorCardProps {
  creator: Creator
  onClick: () => void
}

export default function CreatorCard({ creator, onClick }: CreatorCardProps) {
  const progressPercentage = (parseInt(creator.raised.replace(/[^0-9]/g, '')) / parseInt(creator.goal.replace(/[^0-9]/g, ''))) * 100

  return (
    <motion.div
      className="african-card p-6 cursor-pointer tribal-hover group"
      onClick={onClick}
      whileHover={{ y: -5 }}
      transition={{ duration: 0.3 }}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="w-16 h-16 bg-gradient-to-br from-wakanda-gold-500 to-wakanda-purple-500 rounded-full p-0.5">
              <div className="w-full h-full bg-gray-800 rounded-full flex items-center justify-center text-2xl font-bold text-wakanda-gold-500">
                {creator.name.split(' ').map(n => n[0]).join('')}
              </div>
            </div>
            {creator.verified && (
              <div className="absolute -top-1 -right-1 bg-wakanda-gold-500 rounded-full p-1">
                <CheckCircle className="w-4 h-4 text-black" />
              </div>
            )}
          </div>
          
          <div>
            <h3 className="text-lg font-semibold text-white group-hover:text-wakanda-gold-400 transition-colors">
              {creator.name}
            </h3>
            <div className="flex items-center gap-1 text-sm text-gray-400">
              <MapPin className="w-3 h-3" />
              {creator.location}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-1 text-sm">
          <Star className="w-4 h-4 text-wakanda-gold-500 fill-current" />
          <span className="text-wakanda-gold-400">{creator.rating}</span>
        </div>
      </div>

      {/* Specialty */}
      <div className="cultural-badge text-xs mb-4">
        {creator.specialty}
      </div>

      {/* Description */}
      <p className="text-gray-300 text-sm mb-4 line-clamp-2">
        {creator.description}
      </p>

      {/* Current Project */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <h4 className="font-medium text-white">{creator.currentProject}</h4>
          <ExternalLink className="w-4 h-4 text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity" />
        </div>
        
        {/* Progress Bar */}
        <div className="relative">
          <div className="w-full bg-gray-700 rounded-full h-2 mb-2">
            <motion.div
              className="bg-gradient-to-r from-wakanda-gold-500 to-wakanda-purple-500 h-2 rounded-full"
              initial={{ width: 0 }}
              animate={{ width: `${progressPercentage}%` }}
              transition={{ duration: 1, delay: 0.5 }}
            />
          </div>
          <div className="flex justify-between text-xs text-gray-400">
            <span>{creator.raised} raised</span>
            <span>{creator.goal} goal</span>
          </div>
        </div>
      </div>

      {/* Stats */}
      <div className="flex items-center justify-between pt-4 border-t border-gray-700">
        <div className="flex items-center gap-1 text-sm text-gray-400">
          <Users className="w-4 h-4" />
          <span>{creator.followers.toLocaleString()} followers</span>
        </div>
        
        <motion.button
          className="flex items-center gap-1 text-sm text-wakanda-gold-400 hover:text-wakanda-gold-300"
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          onClick={(e) => {
            e.stopPropagation()
            // Handle support action
          }}
        >
          <Heart className="w-4 h-4" />
          Support
        </motion.button>
      </div>

      {/* Achievements Preview */}
      {creator.achievements.length > 0 && (
        <div className="mt-4 pt-4 border-t border-gray-700">
          <div className="flex flex-wrap gap-1">
            {creator.achievements.slice(0, 2).map((achievement, index) => (
              <span
                key={index}
                className="px-2 py-1 bg-wakanda-purple-500/20 text-wakanda-purple-300 text-xs rounded-full"
              >
                {achievement}
              </span>
            ))}
            {creator.achievements.length > 2 && (
              <span className="px-2 py-1 bg-gray-700 text-gray-400 text-xs rounded-full">
                +{creator.achievements.length - 2} more
              </span>
            )}
          </div>
        </div>
      )}
    </motion.div>
  )
}