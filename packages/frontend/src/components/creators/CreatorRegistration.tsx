'use client'

import { motion } from 'framer-motion'
import { useState } from 'react'
import { Upload, User, MapPin, Palette, FileText, Zap } from 'lucide-react'

export default function CreatorRegistration() {
  const [formData, setFormData] = useState({
    name: '',
    location: '',
    specialty: '',
    bio: '',
    portfolio: '',
    culturalBackground: '',
  })

  const specialties = [
    'Music Producer',
    'Film Director',
    'Digital Artist',
    'Writer/Author',
    'Fashion Designer',
    'Traditional Craftsperson',
    'Photographer',
    'Dancer/Choreographer',
    'Other',
  ]

  const culturalRegions = [
    'West Africa',
    'East Africa',
    'North Africa',
    'Central Africa',
    'Southern Africa',
    'African Diaspora',
  ]

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // Handle form submission
    console.log('Creator registration:', formData)
  }

  return (
    <div className="max-w-2xl mx-auto">
      <motion.div
        className="text-center mb-8"
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <h2 className="text-3xl font-heading font-bold text-gradient mb-4">
          Join the Collective
        </h2>
        <p className="text-gray-300">
          Become part of Africa's premier creative community and start funding your projects today
        </p>
      </motion.div>

      <motion.form
        onSubmit={handleSubmit}
        className="african-card p-8 space-y-6"
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.2 }}
      >
        {/* Personal Information */}
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-white flex items-center gap-2">
            <User className="w-5 h-5 text-wakanda-gold-500" />
            Personal Information
          </h3>
          
          <div className="grid md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Full Name
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({...formData, name: e.target.value})}
                className="w-full px-4 py-3 bg-black/30 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-wakanda-gold-500 focus:outline-none transition-colors"
                placeholder="Enter your full name"
                required
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Location
              </label>
              <div className="relative">
                <MapPin className="absolute left-3 top-3 w-5 h-5 text-gray-400" />
                <input
                  type="text"
                  value={formData.location}
                  onChange={(e) => setFormData({...formData, location: e.target.value})}
                  className="w-full pl-10 pr-4 py-3 bg-black/30 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-wakanda-gold-500 focus:outline-none transition-colors"
                  placeholder="City, Country"
                  required
                />
              </div>
            </div>
          </div>
        </div>

        {/* Creative Information */}
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-white flex items-center gap-2">
            <Palette className="w-5 h-5 text-wakanda-gold-500" />
            Creative Profile
          </h3>
          
          <div className="grid md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Specialty
              </label>
              <select
                value={formData.specialty}
                onChange={(e) => setFormData({...formData, specialty: e.target.value})}
                className="w-full px-4 py-3 bg-black/30 border border-gray-600 rounded-lg text-white focus:border-wakanda-gold-500 focus:outline-none transition-colors"
                required
              >
                <option value="">Select your specialty</option>
                {specialties.map((specialty) => (
                  <option key={specialty} value={specialty}>
                    {specialty}
                  </option>
                ))}
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Cultural Background
              </label>
              <select
                value={formData.culturalBackground}
                onChange={(e) => setFormData({...formData, culturalBackground: e.target.value})}
                className="w-full px-4 py-3 bg-black/30 border border-gray-600 rounded-lg text-white focus:border-wakanda-gold-500 focus:outline-none transition-colors"
                required
              >
                <option value="">Select your region</option>
                {culturalRegions.map((region) => (
                  <option key={region} value={region}>
                    {region}
                  </option>
                ))}
              </select>
            </div>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Bio
            </label>
            <textarea
              value={formData.bio}
              onChange={(e) => setFormData({...formData, bio: e.target.value})}
              rows={4}
              className="w-full px-4 py-3 bg-black/30 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-wakanda-gold-500 focus:outline-none transition-colors resize-none"
              placeholder="Tell us about your creative journey and cultural background..."
              required
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Portfolio URL
            </label>
            <input
              type="url"
              value={formData.portfolio}
              onChange={(e) => setFormData({...formData, portfolio: e.target.value})}
              className="w-full px-4 py-3 bg-black/30 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-wakanda-gold-500 focus:outline-none transition-colors"
              placeholder="https://your-portfolio.com"
            />
          </div>
        </div>

        {/* Portfolio Upload */}
        <div className="space-y-4">
          <h3 className="text-xl font-semibold text-white flex items-center gap-2">
            <Upload className="w-5 h-5 text-wakanda-gold-500" />
            Portfolio Upload
          </h3>
          
          <motion.div
            className="border-2 border-dashed border-gray-600 rounded-lg p-8 text-center hover:border-wakanda-gold-500 transition-colors cursor-pointer"
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <Upload className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-300 mb-2">
              Drop your portfolio files here or click to browse
            </p>
            <p className="text-sm text-gray-500">
              Support for images, videos, audio files (Max 50MB)
            </p>
          </motion.div>
        </div>

        {/* Verification Info */}
        <div className="bg-wakanda-purple-500/10 border border-wakanda-purple-500/30 rounded-lg p-4">
          <div className="flex items-start gap-3">
            <FileText className="w-5 h-5 text-wakanda-purple-400 mt-0.5" />
            <div>
              <h4 className="font-medium text-wakanda-purple-300 mb-1">
                Cultural Verification Process
              </h4>
              <p className="text-sm text-gray-400">
                Our community validators will review your profile to ensure cultural authenticity. 
                This process typically takes 24-48 hours and helps maintain the integrity of our platform.
              </p>
            </div>
          </div>
        </div>

        {/* Submit Button */}
        <motion.button
          type="submit"
          className="w-full btn-african text-lg py-4 flex items-center justify-center gap-2"
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
        >
          <Zap className="w-5 h-5" />
          Join AfroCreate Collective
        </motion.button>
      </motion.form>
    </div>
  )
}