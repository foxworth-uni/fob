#!/usr/bin/env ruby
# frozen_string_literal: true

# Advanced Fob Bundler Example
#
# Demonstrates advanced features like:
# - Multiple entry points
# - Code splitting
# - Library mode
# - Component library mode
# - MDX support

# Load the native extension
# In a real gem setup, this would be: require 'fob'
# For development, we try multiple paths
begin
  require 'fob'
rescue LoadError
  # Try loading from debug build
  debug_path = File.expand_path('../../../../crates/fob-ruby/target/debug/libfob', __FILE__)
  if File.exist?(debug_path)
    require debug_path
  else
    # Try release build
    release_path = File.expand_path('../../../../crates/fob-ruby/target/release/libfob', __FILE__)
    if File.exist?(release_path)
      require release_path
    else
      puts "❌ Error: fob module not found!"
      puts "\nTo build the extension:"
      puts "  cd ../../.."
      puts "  cargo build --package fob-ruby"
      puts "\nOr for release build:"
      puts "  cargo build --release --package fob-ruby"
      exit 1
    end
  end
end

def example_bundle_entry
  puts "\n=== Example 1: Bundle Entry (Single File) ==="
  
  result = Fob.bundle_entry(
    'src/index.js',
    {
      out_dir: 'dist/bundle_entry',
      format: :esm,
      sourcemap: 'external',
      minify: false
    }
  )
  
  puts "✅ Bundled #{result[:stats][:total_modules]} modules"
  puts "   Output: #{result[:chunks].first[:file_name]}"
end

def example_library
  puts "\n=== Example 2: Library Mode (Externalize Dependencies) ==="
  
  result = Fob.library(
    'src/index.js',
    {
      out_dir: 'dist/library',
      external: ['react', 'react-dom'],
      format: :esm
    }
  )
  
  puts "✅ Library built"
  puts "   External dependencies will not be bundled"
end

def example_app
  puts "\n=== Example 3: App Mode (Code Splitting) ==="
  
  # Note: This requires multiple entry files
  # For demo purposes, we'll use the same file twice
  result = Fob.app(
    ['src/index.js', 'src/utils.js'],
    {
      out_dir: 'dist/app',
      code_splitting: {
        min_size: 1000,
        min_imports: 1
      },
      format: :esm
    }
  )
  
  puts "✅ App built with code splitting"
  puts "   Generated #{result[:stats][:total_chunks]} chunks"
end

def example_components
  puts "\n=== Example 4: Component Library Mode ==="
  
  result = Fob.components(
    ['src/index.js', 'src/utils.js'],
    {
      out_dir: 'dist/components',
      format: :esm,
      external_from_manifest: true
    }
  )
  
  puts "✅ Component library built"
  puts "   Each entry produces a separate bundle"
end

def example_full_config
  puts "\n=== Example 5: Full Configuration ==="
  
  bundler = Fob::Bundler.new(
    entries: ['src/index.js'],
    out_dir: 'dist/full_config',
    format: :esm,
    sourcemap: 'inline',
    platform: 'browser',
    minify: true,
    external: ['lodash'],
    entry_mode: :shared,
    cwd: Dir.pwd
  )
  
  result = bundler.bundle
  
  puts "✅ Full configuration bundling complete"
  puts "   Minified: true"
  puts "   Platform: browser"
  puts "   Source maps: inline"
end

def example_mdx
  puts "\n=== Example 6: MDX Support ==="
  
  # Note: This requires an .mdx file
  # For demo, we'll show the configuration
  puts "MDX configuration example:"
  puts <<~CONFIG
    bundler = Fob::Bundler.new(
      entries: ['src/post.mdx'],
      mdx: {
        gfm: true,
        footnotes: true,
        math: true,
        jsx_runtime: 'react/jsx-runtime',
        use_default_plugins: true
      }
    )
  CONFIG
end

def main
  puts "🚀 Advanced Fob Bundler Examples\n"
  
  Fob.init_logging(:info)
  
  begin
    example_bundle_entry
    example_library
    example_app
    example_components
    example_full_config
    example_mdx
    
    puts "\n✅ All examples completed!"
    puts "\n💡 Check the dist/ directory for outputs"
    
  rescue Fob::Error => e
    puts "\n❌ Error: #{e.message}"
    exit 1
  rescue StandardError => e
    puts "\n❌ Unexpected error: #{e.message}"
    puts e.backtrace.first(5)
    exit 1
  end
end

main if __FILE__ == $PROGRAM_NAME
