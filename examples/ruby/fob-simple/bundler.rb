#!/usr/bin/env ruby
# frozen_string_literal: true

# Simple Fob Bundler Example
#
# This demonstrates the most basic way to use Fob from Ruby.
# Perfect for getting started with JavaScript bundling in Ruby!

# Load the native extension
# In a real gem setup, this would be: require 'fob'
# For development, we try multiple paths
begin
  require 'fob_ruby'
rescue LoadError
  # Try loading from workspace target directory (cargo workspace builds here)
  # Ruby on macOS needs .bundle extension, on Linux .so
  # The init function is Init_fob_ruby (based on package name)
  target_dir = File.expand_path('../../../../target', __FILE__)
  debug_bundle = File.join(target_dir, 'debug', 'fob_ruby.bundle')
  release_bundle = File.join(target_dir, 'release', 'fob_ruby.bundle')
  debug_so = File.join(target_dir, 'debug', 'fob_ruby.so')
  release_so = File.join(target_dir, 'release', 'fob_ruby.so')

  if File.exist?(debug_bundle)
    require debug_bundle
  elsif File.exist?(release_bundle)
    require release_bundle
  elsif File.exist?(debug_so)
    require debug_so
  elsif File.exist?(release_so)
    require release_so
  else
    puts "❌ Error: fob_ruby module not found!"
    puts "\nTo build the extension:"
    puts "  cd ../../.."
    puts "  cargo build --package fob-ruby"
    puts "\nOn macOS, create the .bundle symlink:"
    puts "  ln -sf target/debug/libfob.dylib target/debug/fob_ruby.bundle"
    puts "\nOr for release build:"
    puts "  cargo build --release --package fob-ruby"
    puts "  ln -sf target/release/libfob.dylib target/release/fob_ruby.bundle"
    exit 1
  end
end

def main
  puts "🚀 Building with Fob...\n"

  begin
    # Initialize logging (optional, defaults to info level)
    Fob.init_logging("info")

    # Method 1: Using the simple bundle_entry helper
    # This is the easiest way to bundle a single file
    result = Fob.bundle_entry(
      'src/index.js',
      {
        out_dir: 'dist',
        format: :esm
      }
    )

    # Method 2: Using the Fob::Bundler class for more control
    # Uncomment to try this instead:
    # bundler = Fob::Bundler.new(
    #   entries: ['src/index.js'],
    #   out_dir: 'dist',
    #   format: :esm,
    #   sourcemap: 'external'
    # )
    # result = bundler.bundle

    # Method 3: Using other preset methods
    # Uncomment to try this instead:
    # result = Fob.library(
    #   'src/index.js',
    #   { out_dir: 'dist', external: ['react', 'react-dom'] }
    # )

    # Display results
    puts "✅ Build complete!\n"

    puts "📦 Chunks generated:"
    result[:chunks].each do |chunk|
      puts "  - #{chunk[:file_name]} (#{chunk[:size]} bytes)"
    end

    puts "\n📊 Build stats:"
    stats = result[:stats]
    puts "  Modules: #{stats[:total_modules]}"
    puts "  Chunks: #{stats[:total_chunks]}"
    puts "  Total size: #{stats[:total_size]} bytes"
    puts "  Duration: #{stats[:duration_ms]}ms"
    puts "  Cache hit rate: #{(stats[:cache_hit_rate] * 100).round(1)}%"

    unless result[:assets].empty?
      puts "\n📁 Assets:"
      result[:assets].each do |asset|
        puts "  - #{asset[:relative_path]} (#{asset[:size]} bytes)"
      end
    end

    puts "\n💡 Output written to: dist/"
    puts "\n🚀 Try it:"
    puts "   node dist/index.js"

  rescue Fob::Error => e
    puts "❌ Build failed: #{e.message}"
    exit 1
  rescue StandardError => e
    puts "❌ Unexpected error: #{e.message}"
    puts e.backtrace
    exit 1
  end
end

main if __FILE__ == $PROGRAM_NAME
