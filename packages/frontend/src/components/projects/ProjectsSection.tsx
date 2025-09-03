'use client'

import { motion } from 'framer-motion'
import { useState } from 'react'
import { Play, DollarSign, Users, Clock, TrendingUp, Filter, Search } from 'lucide-react'
import ProjectCard from './ProjectCard'

export default function ProjectsSection() {
  const [activeFilter, setActiveFilter] = useState('all')
  const [searchTerm, setSearchTerm] = useState('')

  const projects = [
    {
      id: 1,
      title: 'Rhythms of Lagos',
      creator: 'Kemi Adebayo',
      category: 'Music',
      description: 'An immersive Afrobeats album capturing the essence of modern Lagos street culture',
      image: '/projects/rhythms-lagos.jpg',
      raised: 12500,
      goal: 15000,
      backers: 124,
      timeLeft: '12 days',
      status: 'active',
      culturalRegion: 'West Africa',
      tags: ['Afrobeats', 'Culture', 'Urban'],
    },
    {
      id: 2,
      title: 'Maasai Futures',
      creator: 'Jelani Mwangi',
      category: 'Digital Art',
      description: 'Digital art collection merging traditional Maasai culture with futuristic concepts',
      image: '/projects/maasai-futures.jpg',
      raised: 18200,
      goal: 25000,
      backers: 89,
      timeLeft: '18 days',
      status: 'active',
      culturalRegion: 'East Africa',
      tags: ['NFT', 'Traditional', 'Future'],
    },
    {
      id: 3,
      title: 'Stories from the Volta',
      creator: 'Amara Okonkwo',
      category: 'Film',
      description: 'Documentary series exploring the rich cultural heritage of the Volta Region',
      image: '/projects/volta-stories.jpg',
      raised: 8750,
      goal: 20000,
      backers: 67,
      timeLeft: '25 days',
      status: 'active',
      culturalRegion: 'West Africa',
      tags: ['Documentary', 'Heritage', 'Stories'],
    },
    {
      id: 4,
      title: 'Sahara Soundscapes',
      creator: 'Amina Touré',
      category: 'Music',
      description: 'Ambient music inspired by the vastness and mystery of the Sahara Desert',
      image: '/projects/sahara-sounds.jpg',
      raised: 15000,
      goal: 12000,
      backers: 156,
      timeLeft: 'Funded',
      status: 'funded',
      culturalRegion: 'North Africa',
      tags: ['Ambient', 'Desert', 'Meditation'],
    },
  ]

  const filters = [
    { id: 'all', label: 'All Projects' },
    { id: 'active', label: 'Active' },
    { id: 'funded', label: 'Funded' },
    { id: 'music', label: 'Music' },
    { id: 'film', label: 'Film' },
    { id: 'art', label: 'Digital Art' },
  ]

  const filteredProjects = projects.filter(project => {
    const matchesFilter = activeFilter === 'all' || 
                         project.status === activeFilter || 
                         project.category.toLowerCase().includes(activeFilter)
    const matchesSearch = project.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         project.creator.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         project.description.toLowerCase().includes(searchTerm.toLowerCase())
    return matchesFilter && matchesSearch
  })

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
            <span className="text-gradient">Cultural Projects</span>
          </h2>
          <p className="text-xl text-gray-300 max-w-3xl mx-auto">
            Discover and fund innovative projects that celebrate African heritage and push creative boundaries
          </p>
        </motion.div>

        {/* Search and Filters */}
        <motion.div
          className="flex flex-col md:flex-row gap-4 mb-8"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2 }}
        >
          {/* Search */}
          <div className="relative flex-1">
            <Search className="absolute left-3 top-3 w-5 h-5 text-gray-400" />
            <input
              type="text"
              placeholder="Search projects, creators, or keywords..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-10 pr-4 py-3 bg-black/30 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-wakanda-gold-500 focus:outline-none transition-colors"
            />
          </div>

          {/* Filters */}
          <div className="flex gap-2 overflow-x-auto">
            {filters.map((filter) => (
              <motion.button
                key={filter.id}
                onClick={() => setActiveFilter(filter.id)}
                className={`px-4 py-2 rounded-lg font-medium whitespace-nowrap transition-all duration-300 ${
                  activeFilter === filter.id
                    ? 'bg-wakanda-gold-500 text-black'
                    : 'bg-black/30 text-gray-300 hover:text-white border border-gray-600 hover:border-wakanda-gold-500'
                }`}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
              >
                {filter.label}
              </motion.button>
            ))}
          </div>
        </motion.div>

        {/* Projects Grid */}
        <motion.div
          className="grid md:grid-cols-2 lg:grid-cols-3 gap-8 mb-12"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 0.4 }}
        >
          {filteredProjects.map((project, index) => (
            <motion.div
              key={project.id}
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.6 + index * 0.1 }}
            >
              <ProjectCard project={project} />
            </motion.div>
          ))}
        </motion.div>

        {/* Project Stats */}
        <motion.div
          className="grid grid-cols-2 md:grid-cols-4 gap-8"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.8 }}
        >
          {[
            { icon: Play, label: 'Projects Launched', value: '890+' },
            { icon: DollarSign, label: 'Total Funded', value: '$1.2M+' },
            { icon: Users, label: 'Community Backers', value: '15.4K+' },
            { icon: TrendingUp, label: 'Success Rate', value: '87%' },
          ].map((stat, index) => (
            <motion.div
              key={stat.label}
              className="text-center african-card p-6"
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 1.0 + index * 0.1 }}
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

        {/* Call to Action */}
        <motion.div
          className="text-center mt-16"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 1.2 }}
        >
          <h3 className="text-2xl font-heading font-bold text-white mb-4">
            Ready to Launch Your Project?
          </h3>
          <p className="text-gray-300 mb-6 max-w-2xl mx-auto">
            Join thousands of African creators who have successfully funded their cultural projects through our platform
          </p>
          <motion.button
            className="btn-african text-lg px-8 py-4"
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            Start Your Project
          </motion.button>
        </motion.div>
      </div>
    </div>
  )
}