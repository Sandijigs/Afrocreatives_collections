'use client'

import { motion } from 'framer-motion'
import { Users, Clock, MapPin, Heart, ExternalLink, CheckCircle } from 'lucide-react'

interface Project {
  id: number
  title: string
  creator: string
  category: string
  description: string
  image: string
  raised: number
  goal: number
  backers: number
  timeLeft: string
  status: string
  culturalRegion: string
  tags: string[]
}

interface ProjectCardProps {
  project: Project
}

export default function ProjectCard({ project }: ProjectCardProps) {
  const progressPercentage = (project.raised / project.goal) * 100
  const isCompleted = project.status === 'funded'

  return (
    <motion.div
      className="african-card overflow-hidden cursor-pointer tribal-hover group"
      whileHover={{ y: -5 }}
      transition={{ duration: 0.3 }}
    >
      {/* Project Image */}
      <div className="relative h-48 bg-gradient-to-br from-wakanda-purple-500/20 to-wakanda-gold-500/20 overflow-hidden">
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-20 h-20 bg-wakanda-gold-500/30 rounded-full flex items-center justify-center">
            <span className="text-2xl font-bold text-wakanda-gold-400">
              {project.title.split(' ').map(w => w[0]).join('')}
            </span>
          </div>
        </div>
        
        {/* Status Badge */}
        <div className="absolute top-4 left-4">
          <span className={`px-3 py-1 rounded-full text-xs font-medium ${
            isCompleted
              ? 'bg-green-500/20 text-green-400'
              : 'bg-blue-500/20 text-blue-400'
          }`}>
            {isCompleted ? 'Funded' : 'Active'}
          </span>
        </div>

        {/* Cultural Region Badge */}
        <div className="absolute top-4 right-4">
          <span className="px-2 py-1 bg-wakanda-purple-500/20 text-wakanda-purple-300 text-xs rounded-full">
            {project.culturalRegion}
          </span>
        </div>

        {/* Hover Overlay */}
        <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center">
          <ExternalLink className="w-8 h-8 text-wakanda-gold-400" />
        </div>
      </div>

      {/* Content */}
      <div className="p-6">
        {/* Header */}
        <div className="mb-4">
          <h3 className="text-lg font-semibold text-white group-hover:text-wakanda-gold-400 transition-colors mb-1">
            {project.title}
          </h3>
          <p className="text-sm text-gray-400">
            by <span className="text-wakanda-gold-400">{project.creator}</span>
          </p>
        </div>

        {/* Category */}
        <div className="cultural-badge text-xs mb-3">
          {project.category}
        </div>

        {/* Description */}
        <p className="text-gray-300 text-sm mb-4 line-clamp-2">
          {project.description}
        </p>

        {/* Tags */}
        <div className="flex flex-wrap gap-1 mb-4">
          {project.tags.slice(0, 3).map((tag, index) => (
            <span
              key={index}
              className="px-2 py-1 bg-gray-700 text-gray-400 text-xs rounded"
            >
              {tag}
            </span>
          ))}
        </div>

        {/* Progress Bar */}
        <div className="mb-4">
          <div className="flex justify-between text-sm mb-2">
            <span className="text-wakanda-gold-400 font-medium">
              ${project.raised.toLocaleString()}
            </span>
            <span className="text-gray-400">
              ${project.goal.toLocaleString()} goal
            </span>
          </div>
          
          <div className="relative">
            <div className="w-full bg-gray-700 rounded-full h-2">
              <motion.div
                className={`h-2 rounded-full ${
                  isCompleted
                    ? 'bg-gradient-to-r from-green-500 to-green-400'
                    : 'bg-gradient-to-r from-wakanda-gold-500 to-wakanda-purple-500'
                }`}
                initial={{ width: 0 }}
                animate={{ width: `${Math.min(progressPercentage, 100)}%` }}
                transition={{ duration: 1, delay: 0.5 }}
              />
            </div>
            {isCompleted && (
              <div className="absolute -top-1 right-0">
                <CheckCircle className="w-4 h-4 text-green-400" />
              </div>
            )}
          </div>
          
          <div className="text-xs text-gray-500 mt-1">
            {Math.round(progressPercentage)}% funded
          </div>
        </div>

        {/* Stats */}
        <div className="flex items-center justify-between pt-4 border-t border-gray-700">
          <div className="flex items-center gap-4 text-sm text-gray-400">
            <div className="flex items-center gap-1">
              <Users className="w-4 h-4" />
              <span>{project.backers}</span>
            </div>
            <div className="flex items-center gap-1">
              <Clock className="w-4 h-4" />
              <span>{project.timeLeft}</span>
            </div>
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
            {isCompleted ? 'View' : 'Back'}
          </motion.button>
        </div>
      </div>
    </motion.div>
  )
}