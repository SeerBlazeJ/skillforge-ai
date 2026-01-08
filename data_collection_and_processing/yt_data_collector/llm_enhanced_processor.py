"""
Enhanced YouTube Data Processor with LLM-powered Analysis
Uses OpenRouter API with FREE models for better topic extraction and prerequisites
"""

import json
import os
import time
from typing import List, Dict, Optional
import requests
from dataclasses import dataclass

@dataclass
class LLMConfig:
    """Configuration for OpenRouter API"""
    api_key: str
    base_url: str = "https://openrouter.ai/api/v1/chat/completions"

    # FREE models on OpenRouter (as of Jan 2026)
    models = {
        "fast": "nvidia/nemotron-3-nano-30b-a3b:free",  # Fast for simple tasks
        "balanced": "mistralai/devstral-2512:free",  # Good balance
        "smart": "xiaomi/mimo-v2-flash:free",  # Good reasoning
        "vision": "mistralai/pixtral-12b:free",  # For visual content
        "long_context": "google/gemini-pro-1.5:free"  # 1M tokens
    }


class LLMEnhancedProcessor:
    def __init__(self, input_file: str, openrouter_api_key: str):
        self.input_file = input_file
        self.llm_config = LLMConfig(api_key=openrouter_api_key)
        self.data = self._load_data()
        self.videos = self.data.get('videos', [])

        # Cache for LLM responses to avoid duplicate calls
        self.topic_cache = {}
        self.prereq_cache = {}

    def _load_data(self) -> Dict:
        """Load JSON data"""
        with open(self.input_file, 'r', encoding='utf-8') as f:
            return json.load(f)

    def _call_llm(self, prompt: str, model_type: str = "balanced",
                  max_tokens: int = 500, temperature: float = 0.25) -> Optional[str]:
        """Call OpenRouter API with free models"""

        model = self.llm_config.models[model_type]

        headers = {
            "Authorization": f"Bearer {self.llm_config.api_key}",
            "HTTP-Referer": "https://github.com/your-repo",  # Optional
            "X-Title": "Learning Roadmap Generator",  # Optional
            "Content-Type": "application/json"
        }

        payload = {
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert educational content analyzer. "
                              "Provide concise, accurate responses in the exact requested format, which should only be based on the context you are given."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": temperature,
            "max_tokens": max_tokens
        }

        try:
            response = requests.post(
                self.llm_config.base_url,
                headers=headers,
                json=payload,
                timeout=30
            )
            response.raise_for_status()

            result = response.json()
            return result['choices'][0]['message']['content'].strip()

        except Exception as e:
            print(f"LLM API error: {e}")
            return None

    def extract_better_topic(self, title: str, description: str,
                            skill_path: str) -> str:
        """Use LLM to extract meaningful topic name"""

        # Create cache key
        cache_key = f"{title}_{skill_path}"
        if cache_key in self.topic_cache:
            return self.topic_cache[cache_key]

        prompt = f"""Extract a concise, meaningful topic name from this educational video.

Title: {title}
Domain: {skill_path}
Description: {description[:200]}

Requirements:
1. Use 2-4 words maximum
2. Use underscores between words
3. Make it searchable and descriptive
4. Focus on the main concept being taught
5. Use technical terms when appropriate

Examples:
- "Learn Python Variables" → "python_variables"
- "Understanding React Hooks Tutorial" → "react_hooks"
- "Deep Dive into Neural Networks" → "neural_networks_fundamentals"

Return ONLY the topic name, nothing else."""

        result = self._call_llm(prompt, model_type="smart", max_tokens=50)

        if result:
            # Clean the response
            topic = result.strip().lower()
            topic = topic.replace(" ", "_").replace("-", "_")
            # Remove any quotes or extra characters
            topic = ''.join(c for c in topic if c.isalnum() or c == '_')
            self.topic_cache[cache_key] = topic
            return topic
        else:
            # Fallback to original method
            return "_".join(title.split()[:3]).lower()

    def extract_robust_prerequisites(self, title: str, description: str,
                                    content: str, skill_path: str,
                                    level: str) -> List[str]:
        """Use LLM to detect comprehensive prerequisites"""

        # Create cache key
        cache_key = f"{title}_{skill_path}_{level}"
        if cache_key in self.prereq_cache:
            return self.prereq_cache[cache_key]

        # Limit content length to avoid token limits
        full_context = f"{title}. {description}. {content[:500]}"

        prompt = f"""Identify the prerequisite knowledge needed to understand this educational content.

Title: {title}
Domain: {skill_path}
Level: {level}
Context: {full_context}

Analyze what concepts, skills, or knowledge a learner should have BEFORE taking this content.

Return a JSON array of prerequisites (3-6 items max). Each should be:
- A single concept/skill (2-4 words)
- Use underscores between words
- Be specific and technical
- Appropriate for the domain

Example outputs:
For "React Hooks Advanced Patterns":
["react_basics", "javascript_es6", "component_lifecycle", "state_management"]

For "Machine Learning with TensorFlow":
["python_programming", "linear_algebra", "calculus_basics", "numpy_pandas"]

For beginner content, return fewer or no prerequisites.
Return ONLY the JSON array, nothing else."""

        result = self._call_llm(
            prompt,
            model_type="smart",  # Use smarter model for this
            max_tokens=200,
            temperature=0.2  # Lower temp for consistent output
        )

        if result:
            try:
                # Try to parse JSON
                import re
                # Extract JSON array if embedded in text
                json_match = re.search(r'\[.*?\]', result, re.DOTALL)
                if json_match:
                    prereqs = json.loads(json_match.group())
                    # Clean and validate
                    prereqs = [p.strip().lower().replace(" ", "_")
                              for p in prereqs if isinstance(p, str)]
                    prereqs = prereqs[:6]  # Limit to 6
                    self.prereq_cache[cache_key] = prereqs
                    return prereqs
            except:
                pass

        # Fallback: return empty or basic prerequisites
        return []

    def enhance_video_data(self, video: Dict) -> Dict:
        """Enhance a single video with LLM analysis"""

        print(f"Enhancing: {video['video_id']} - {video.get('topic', 'unknown')}")

        # Extract better topic
        better_topic = self.extract_better_topic(
            video.get('title', video.get('topic', '')),
            video.get('description', video.get('content', '')[:200]),
            video.get('skill_path', 'general')
        )

        # Extract robust prerequisites
        robust_prereqs = self.extract_robust_prerequisites(
            video.get('title', video.get('topic', '')),
            video.get('description', video.get('content', '')[:300]),
            video.get('content', ''),
            video.get('skill_path', 'general'),
            video.get('level', 'intermediate')
        )

        # Update video data
        video['topic'] = better_topic
        video['prerequisite_topics'] = robust_prereqs

        # Add LLM metadata
        video['enhanced_with_llm'] = True

        return video

    def enhance_all_videos(self, batch_size: int = 10):
        """Process all videos in batches to avoid rate limits"""

        print(f"\nEnhancing {len(self.videos)} videos with LLM analysis...")
        print("Using FREE OpenRouter models:")
        print(f"  - Fast: {self.llm_config.models['fast']}")
        print(f"  - Smart: {self.llm_config.models['smart']}")

        enhanced_videos = []

        for i, video in enumerate(self.videos):
            try:
                enhanced_video = self.enhance_video_data(video)
                enhanced_videos.append(enhanced_video)

                # Rate limiting: pause after each batch
                if (i + 1) % batch_size == 0:
                    print(f"  Processed {i + 1}/{len(self.videos)} videos... pausing...")
                    time.sleep(2)  # 2 second pause between batches

            except Exception as e:
                print(f"  Error enhancing video {video.get('video_id')}: {e}")
                enhanced_videos.append(video)  # Keep original if enhancement fails

        self.videos = enhanced_videos
        print(f"✓ Enhanced {len(enhanced_videos)} videos")

    def add_learning_pathways(self):
        """Use LLM to suggest learning pathways between topics"""

        print("\nGenerating learning pathway suggestions...")

        # Group videos by domain
        domain_videos = {}
        for video in self.videos:
            domain = video.get('skill_path', 'general')
            if domain not in domain_videos:
                domain_videos[domain] = []
            domain_videos[domain].append(video)

        pathways = {}

        for domain, videos in domain_videos.items():
            # Get all topics in this domain
            topics = [v['topic'] for v in videos]

            if len(topics) < 3:
                continue  # Skip domains with too few videos

            prompt = f"""Create a learning pathway for the domain: {domain}

Available topics:
{', '.join(topics[:20])}

Suggest a logical learning sequence. Return a JSON object like:
{{
  "beginner_path": ["topic1", "topic2", "topic3"],
  "intermediate_path": ["topic4", "topic5"],
  "advanced_path": ["topic6"]
}}

Only include topics from the list above. Group by difficulty level."""

            result = self._call_llm(
                prompt,
                model_type="smart",
                max_tokens=300
            )

            if result:
                try:
                    import re
                    json_match = re.search(r'\{.*?\}', result, re.DOTALL)
                    if json_match:
                        pathway = json.loads(json_match.group())
                        pathways[domain] = pathway
                except:
                    pass

        return pathways

    def export_enhanced_data(self, output_file: str = "enhanced_learning_data.json"):
        """Export enhanced data with LLM improvements"""

        print(f"\nExporting enhanced data to {output_file}...")

        # Generate pathways
        pathways = self.add_learning_pathways()

        # Build output
        output = {
            "metadata": {
                "total_videos": len(self.videos),
                "enhanced_with_llm": True,
                "llm_models_used": {
                    "topic_extraction": self.llm_config.models["fast"],
                    "prerequisite_detection": self.llm_config.models["smart"]
                },
                "processing_date": self.data.get('metadata', {}).get('processing_date'),
                "original_statistics": self.data.get('metadata', {}).get('statistics')
            },
            "videos": self.videos,
            "learning_pathways": pathways
        }

        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(output, f, indent=2, ensure_ascii=False)

        print(f"✓ Exported {len(self.videos)} enhanced videos")
        print(f"✓ Generated {len(pathways)} learning pathways")

        return output

    def generate_quality_report(self):
        """Generate report on data quality improvements"""

        print("\n" + "="*70)
        print("ENHANCEMENT QUALITY REPORT")
        print("="*70)

        # Count videos with prerequisites
        with_prereqs = sum(1 for v in self.videos if v.get('prerequisite_topics'))
        avg_prereqs = sum(len(v.get('prerequisite_topics', [])) for v in self.videos) / len(self.videos)

        # Count enhanced videos
        enhanced = sum(1 for v in self.videos if v.get('enhanced_with_llm'))

        print(f"\nVideos Enhanced: {enhanced}/{len(self.videos)} ({enhanced/len(self.videos)*100:.1f}%)")
        print(f"Videos with Prerequisites: {with_prereqs}/{len(self.videos)} ({with_prereqs/len(self.videos)*100:.1f}%)")
        print(f"Average Prerequisites per Video: {avg_prereqs:.2f}")

        # Sample improved topics
        print("\nSample Improved Topics:")
        for i, video in enumerate(self.videos[:5]):
            print(f"  {i+1}. {video.get('topic')} ({video.get('skill_path')})")
            if video.get('prerequisite_topics'):
                print(f"     Prerequisites: {', '.join(video['prerequisite_topics'][:3])}")


def main():
    """Main execution"""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python llm_enhanced_processor.py <input_json_file>")
        print("Example: python llm_enhanced_processor.py youtube_educational_data.json")
        print("\nSet OPENROUTER_API_KEY environment variable first!")
        return

    input_file = sys.argv[1]

    # Get OpenRouter API key
    api_key = os.environ.get('OPENROUTER_API_KEY')
    if not api_key:
        print("ERROR: OPENROUTER_API_KEY not set")
        print("Get free API key at: https://openrouter.ai/keys")
        api_key = input("Or enter your API key now: ").strip()

    if not api_key:
        print("No API key provided. Exiting...")
        return

    print("LLM-Enhanced Educational Data Processor")
    print("="*70)
    print("Using FREE OpenRouter models")
    print(f"Input: {input_file}")
    print()

    # Initialize processor
    processor = LLMEnhancedProcessor(input_file, api_key)

    # Enhance all videos
    processor.enhance_all_videos(batch_size=10)

    # Export results
    processor.export_enhanced_data("enhanced_learning_data.json")

    # Generate quality report
    processor.generate_quality_report()

    print("\n" + "="*70)
    print("PROCESSING COMPLETE")
    print("="*70)
    print("\nGenerated files:")
    print("  1. enhanced_learning_data.json - LLM-enhanced data")
    print("\nImprovements:")
    print("  ✓ Better topic names (semantic extraction)")
    print("  ✓ Robust prerequisites (context-aware)")
    print("  ✓ Learning pathways (suggested sequences)")
    print("  ✓ Ready for RAG ingestion")


if __name__ == "__main__":
    main()
