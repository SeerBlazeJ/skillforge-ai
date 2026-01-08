"""
YouTube Educational Scraper with LLM Enhancement
Integrated version that scrapes AND enhances in one run
"""

import json
import os
import time
from typing import List, Dict, Optional
import requests
from datetime import datetime
from googleapiclient.discovery import build
from youtube_transcript_api import YouTubeTranscriptApi

# Import LLM enhancement (if available)
LLM_AVAILABLE = True
try:
    from llm_enhanced_processor import LLMConfig, LLMEnhancedProcessor
except ImportError:
    LLM_AVAILABLE = False


class IntegratedYouTubeScraper:
    """Scrapes YouTube data and optionally enhances with LLM analysis"""

    def __init__(self, api_key: str, openrouter_key: Optional[str] = None):
        self.youtube = build('youtube', 'v3', developerKey=api_key)
        self.openrouter_key = openrouter_key
        self.enable_llm = bool(openrouter_key and LLM_AVAILABLE)

        if self.enable_llm:
            self.llm_config = LLMConfig(api_key=openrouter_key)

        print(f"Scraper initialized | LLM Enhancement: {'✓ Enabled' if self.enable_llm else '✗ Disabled'}")

    def get_channel_videos(self, channel_id: str, max_results: int = 10) -> List[Dict]:
        """Fetch videos from a channel"""
        try:
            # Get uploads playlist ID
            channel_response = self.youtube.channels().list(
                part='contentDetails,snippet',
                id=channel_id
            ).execute()

            if not channel_response['items']:
                return []

            channel_name = channel_response['items'][0]['snippet']['title']
            uploads_playlist_id = channel_response['items'][0]['contentDetails']['relatedPlaylists']['uploads']

            # Get videos from uploads playlist
            videos = []
            playlist_response = self.youtube.playlistItems().list(
                part='contentDetails',
                playlistId=uploads_playlist_id,
                maxResults=max_results
            ).execute()

            video_ids = [item['contentDetails']['videoId'] for item in playlist_response['items']]

            # Get video details
            videos_response = self.youtube.videos().list(
                part='snippet,statistics,contentDetails',
                id=','.join(video_ids)
            ).execute()

            for video in videos_response['items']:
                videos.append({
                    'video_id': video['id'],
                    'title': video['snippet']['title'],
                    'description': video['snippet']['description'],
                    'channel_name': channel_name,
                    'published_date': video['snippet']['publishedAt'],
                    'views': int(video['statistics'].get('viewCount', 0)),
                    'likes': int(video['statistics'].get('likeCount', 0)),
                    'duration': video['contentDetails']['duration']
                })

            return videos

        except Exception as e:
            print(f"Error fetching from {channel_id}: {e}")
            return []

    def get_transcript(self, video_id: str) -> str:
        """Get video transcript"""
        try:
            transcript_list = YouTubeTranscriptApi.get_transcript(video_id)
            transcript = ' '.join([entry['text'] for entry in transcript_list])
            return transcript[:1000]  # Limit length
        except:
            return ""

    def enhance_with_llm(self, video_data: Dict) -> Dict:
        """Enhance single video with LLM analysis"""
        if not self.enable_llm:
            return video_data

        try:
            # Extract better topic
            prompt_topic = f"""Extract a concise topic name (2-4 words, use underscores):
Title: {video_data['title']}
Domain: {video_data.get('skill_path', 'general')}
Return ONLY the topic name."""

            headers = {
                "Authorization": f"Bearer {self.openrouter_key}",
                "Content-Type": "application/json"
            }

            payload = {
                "model": "google/gemma-2-2b-it:free",
                "messages": [{"role": "user", "content": prompt_topic}],
                "temperature": 0.3,
                "max_tokens": 50
            }

            response = requests.post(
                "https://openrouter.ai/api/v1/chat/completions",
                headers=headers,
                json=payload,
                timeout=15
            )

            if response.status_code == 200:
                topic = response.json()['choices'][0]['message']['content'].strip()
                topic = topic.lower().replace(" ", "_").replace("-", "_")
                video_data['topic'] = topic
                video_data['llm_enhanced'] = True

            # Get prerequisites (separate call)
            time.sleep(0.5)  # Rate limit

            prompt_prereq = f"""List prerequisites for this content as JSON array (3-5 items, underscores):
Title: {video_data['title']}
Level: {video_data.get('level', 'intermediate')}
Return ONLY JSON array like: ["python_basics", "algorithms"]"""

            payload['messages'] = [{"role": "user", "content": prompt_prereq}]
            payload['model'] = "google/gemini-2.0-flash-exp:free"
            payload['max_tokens'] = 150

            response = requests.post(
                "https://openrouter.ai/api/v1/chat/completions",
                headers=headers,
                json=payload,
                timeout=15
            )

            if response.status_code == 200:
                prereq_text = response.json()['choices'][0]['message']['content']
                import re
                json_match = re.search(r'\[.*?\]', prereq_text, re.DOTALL)
                if json_match:
                    prereqs = json.loads(json_match.group())
                    video_data['prerequisite_topics'] = [p.lower().replace(" ", "_") for p in prereqs]

        except Exception as e:
            print(f"  LLM enhancement failed for {video_data['video_id']}: {e}")

        return video_data

    def scrape_and_enhance(self, channels: List[Dict], videos_per_channel: int = 10) -> Dict:
        """Main scraping and enhancement pipeline"""

        all_videos = []

        print(f"\nScraping {len(channels)} channels...")

        for i, channel in enumerate(channels):
            print(f"  [{i+1}/{len(channels)}] {channel['name']}...", end=" ")

            videos = self.get_channel_videos(channel['id'], videos_per_channel)

            for video in videos:
                # Add metadata
                video['skill_path'] = channel.get('domain', 'general')
                video['level'] = channel.get('default_level', 'intermediate')
                video['type'] = 'macro' if 'course' in video['title'].lower() else 'micro'

                # Get transcript
                video['content'] = self.get_transcript(video['video_id'])

                # Basic topic extraction (fallback)
                if 'topic' not in video:
                    video['topic'] = "_".join(video['title'].split()[:3]).lower()
                if 'prerequisite_topics' not in video:
                    video['prerequisite_topics'] = []

                # Enhance with LLM if enabled
                if self.enable_llm:
                    video = self.enhance_with_llm(video)
                    time.sleep(0.5)  # Rate limiting

                all_videos.append(video)

            print(f"✓ {len(videos)} videos")
            time.sleep(1)  # Be nice to YouTube API

        # Compile output
        output = {
            "metadata": {
                "total_videos": len(all_videos),
                "processing_date": datetime.now().isoformat(),
                "llm_enhanced": self.enable_llm
            },
            "videos": all_videos
        }

        return output


def main():
    """Example usage"""

    # Get API keys
    youtube_key = os.environ.get('YOUTUBE_API_KEY')
    openrouter_key = os.environ.get('OPENROUTER_API_KEY')  # Optional

    if not youtube_key:
        print("Error: YOUTUBE_API_KEY not set")
        return

    # Example channels
    channels = [
    # --- Programming & Computer Science ---
    {
        "id": "UC8butISFwT-Wl7EV0hUK0BQ",
        "name": "freeCodeCamp.org",
        "domain": "programming_general",
        "default_level": "beginner"
    },
    {
        "id": "UCeVMnSShP_Iviwkknt83cww",
        "name": "Code with Mosh",
        "domain": "programming_general",
        "default_level": "beginner"
    },
    {
        "id": "UC29ju8bIPH5as8OGnQzwJyA",
        "name": "Traversy Media",
        "domain": "programming_general",
        "default_level": "intermediate"
    },
    {
        "id": "UCW5YeuERMmlnqo4oq8vwUpg",
        "name": "Net Ninja",
        "domain": "programming_general",
        "default_level": "intermediate"
    },
    {
        "id": "UCsBjURrPoezykLs9EqgamOA",
        "name": "Fireship",
        "domain": "programming_general",
        "default_level": "intermediate"
    },
    {
        "id": "UC8butISFwT-Wl7EV0hUK0BQ",
        "name": "CS50",
        "domain": "computer_science",
        "default_level": "beginner"
    },
    {
        "id": "UC9-y-6csu5WGm29I7JiwpnA",
        "name": "Computerphile",
        "domain": "computer_science",
        "default_level": "intermediate"
    },
    {
        "id": "UCBa659QWEk1AI4Tg--mrJ2A",
        "name": "Tom Scott",
        "domain": "general_knowledge_engineering",
        "default_level": "beginner"
    },
    {
        "id": "UC6107grRI4m0o2-emgoDnAA",
        "name": "SmarterEveryDay",
        "domain": "general_knowledge_engineering",
        "default_level": "intermediate"
    },
     {
        "id": "UCY1kMZp36EZonL4ALVIbF2g",
        "name": "Mark Rober",
        "domain": "general_knowledge_engineering",
        "default_level": "beginner"
    },

    # --- Python ---
    {
        "id": "UCCezIgC97PvUuR4_gbFUs5g",
        "name": "Corey Schafer",
        "domain": "python_programming",
        "default_level": "intermediate"
    },
    {
        "id": "UCfzlCWGWYyIQ0aLC5w48gBQ",
        "name": "Sentdex",
        "domain": "python_programming",
        "default_level": "intermediate"
    },
    {
        "id": "UCWv7vMbMWH4-V0ZXdmDpPBA",
        "name": "Programming with Mosh",
        "domain": "python_programming",
        "default_level": "beginner"
    },
    {
        "id": "UC4JX40jDee_tINbkjycV4Sg",
        "name": "Tech with Tim",
        "domain": "python_programming",
        "default_level": "beginner"
    },
    {
        "id": "UC0T6Op8vrC6tBVCpG2UTIgw",
        "name": "ArjanCodes",
        "domain": "python_programming",
        "default_level": "advanced"
    },
    {
        "id": "UCx0tMNIVQ-bCXc-eGf_t96Q",
        "name": "mCoding",
        "domain": "python_programming",
        "default_level": "advanced"
    },

    # --- Rust & Systems ---
    {
        "id": "UC_iD0xppBwwsrM9DegC5cQQ",
        "name": "Let's Get Rusty",
        "domain": "rust_programming",
        "default_level": "beginner"
    },
    {
        "id": "UCaYhcUwRBNscFNUKTjgPFiA",
        "name": "No Boilerplate",
        "domain": "rust_programming",
        "default_level": "intermediate"
    },
    {
        "id": "UC_iD0xppBwwsrM9DegC5cQQ",
        "name": "Jon Gjengset",
        "domain": "rust_programming",
        "default_level": "advanced"
    },
    {
        "id": "UC6Vfhlf2C04JDxcLt-ObMvA",
        "name": "Low Level Learning",
        "domain": "low_level_systems",
        "default_level": "intermediate"
    },
    {
        "id": "UClcE-kVhqyiHCcjYwcpfj9w",
        "name": "LiveOverflow",
        "domain": "low_level_systems",
        "default_level": "advanced"
    },

    # --- Web Development ---
    {
        "id": "UCFbNIlppjAuEX4znoulh0Cw",
        "name": "Web Dev Simplified",
        "domain": "javascript_web_dev",
        "default_level": "beginner"
    },
    {
        "id": "UClb90NQQcskPUGDIXsQEz5Q",
        "name": "Dev Ed",
        "domain": "javascript_web_dev",
        "default_level": "beginner"
    },

    # --- Data Science & AI ---
    {
        "id": "UCNU_lfiiWBdtULKOw6X0Dig",
        "name": "Krish Naik",
        "domain": "data_science",
        "default_level": "beginner"
    },
    {
        "id": "UCh9nVJoWXmFb7sLApWGcLPQ",
        "name": "Codebasics",
        "domain": "data_science",
        "default_level": "beginner"
    },
    {
        "id": "UCLLw7jmFsvfIVaUFsLs8mlQ",
        "name": "StatQuest",
        "domain": "data_science",
        "default_level": "intermediate"
    },
    {
        "id": "UCYO_jab_esuFRV4b17AJtAw",
        "name": "3Blue1Brown",
        "domain": "machine_learning_ai",
        "default_level": "intermediate"
    },
    {
        "id": "UCI-HzinCi5os9UlVI2_bcUw",
        "name": "Andrej Karpathy",
        "domain": "machine_learning_ai",
        "default_level": "advanced"
    },
    {
        "id": "UC7_gcs09iThXybpVgjHZ_7g",
        "name": "PBS Space Time",
        "domain": "machine_learning_ai",
        "default_level": "intermediate"
    },
    {
        "id": "UCcIXc5mJsHVYTZR1maL5l9w",
        "name": "DeepLearningAI",
        "domain": "deep_learning",
        "default_level": "intermediate"
    },

    # --- System Design & Backend ---
    {
        "id": "UCZHmQk67mSJgfCCTn7mVfqQ",
        "name": "ByteByteGo",
        "domain": "system_design_backend",
        "default_level": "intermediate"
    },
    {
        "id": "UC_ML5xP23TOWKUcc-oAE_Eg",
        "name": "Hussein Nasser",
        "domain": "system_design_backend",
        "default_level": "advanced"
    },
    {
        "id": "UCZLJf_R2sWyUtXSKiKlyvAw",
        "name": "Abdul Bari",
        "domain": "algorithms_data_structures",
        "default_level": "intermediate"
    },
    {
        "id": "UCZCFT11CWBi3MHNlGf019nw",
        "name": "William Fiset",
        "domain": "algorithms_data_structures",
        "default_level": "intermediate"
    },

    # --- Cyber Security & Cloud ---
    {
        "id": "UC0ArlFuFYMpEewyRBzdLHiw",
        "name": "The Cyber Mentor",
        "domain": "cybersecurity",
        "default_level": "beginner"
    },
    {
        "id": "UC9x0AN7BWHpCDHSm9NiJFJQ",
        "name": "NetworkChuck",
        "domain": "cybersecurity",
        "default_level": "beginner"
    },
    {
        "id": "UCY0xL8V6NzzFcwzHCgB8orQ",
        "name": "Whiteboard Crypto",
        "domain": "blockchain_crypto",
        "default_level": "beginner"
    },
    {
        "id": "UCwKBqkz8_qPmVVTxGAGGkKw",
        "name": "TechWorld with Nana",
        "domain": "cloud_computing",
        "default_level": "beginner"
    },

    # --- Math & Physics ---
    {
        "id": "UCtAIs1VCQrymlAnw3mGonhw",
        "name": "patrickJMT",
        "domain": "mathematics",
        "default_level": "beginner"
    },
    {
        "id": "UCoxcjq-8xIDTYp3uz647V5A",
        "name": "Numberphile",
        "domain": "mathematics",
        "default_level": "beginner"
    },
    {
        "id": "UC1_uAIS3r8Vu6JjXWvastJg",
        "name": "Mathologer",
        "domain": "mathematics",
        "default_level": "intermediate"
    },
    {
        "id": "UC7dBj4l6o_CXEif5r-F-tRA",
        "name": "Stand-up Maths",
        "domain": "mathematics",
        "default_level": "intermediate"
    },
    {
        "id": "UCJ0-OtVpF0wOKEqT2Z1HEtA",
        "name": "MinutePhysics",
        "domain": "physics",
        "default_level": "beginner"
    },
    {
        "id": "UCHnyfMqiRRG1u-2MsSQLbXA",
        "name": "Veritasium",
        "domain": "physics",
        "default_level": "beginner"
    },

    # --- Engineering (Mech/Elec/Civil) ---
    {
        "id": "UCaiL2GDNpLYH6Wokkk1VNcg",
        "name": "GreatScott!",
        "domain": "electronics_embedded",
        "default_level": "intermediate"
    },
    {
        "id": "UCqQgeY4DStmV6VkJkNTjhtA",
        "name": "Andreas Spiess",
        "domain": "electronics_embedded",
        "default_level": "advanced"
    },
    {
        "id": "UCR1IuLEqb6UEA_zQ81kwXfg",
        "name": "Real Engineering",
        "domain": "mechanical_engineering",
        "default_level": "intermediate"
    },
    {
        "id": "UC2bkHVIDjXS7sgrgjFtzOXQ",
        "name": "Lesics",
        "domain": "mechanical_engineering",
        "default_level": "intermediate"
    },
    {
        "id": "UClBLFU_XEkDOu6aH-BMXp4w",
        "name": "The B1M",
        "domain": "civil_engineering",
        "default_level": "beginner"
    },

    # --- Finance & Economics ---
    {
        "id": "UCnMn36GT_H0X-w5_ckLtlgQ",
        "name": "The Plain Bagel",
        "domain": "finance_investing",
        "default_level": "beginner"
    },
    {
        "id": "UCqK_GSMbpiV8spgD3ZGloSw",
        "name": "Ben Felix",
        "domain": "finance_investing",
        "default_level": "intermediate"
    },
    {
        "id": "UCIALMKvObZNtJ6AmdCLP7Lg",
        "name": "Financial Education",
        "domain": "finance_investing",
        "default_level": "beginner"
    },
    {
        "id": "UCm0dGHae5xPZVWZIal2fX7w",
        "name": "Dimitri Bianco",
        "domain": "quantitative_finance",
        "default_level": "advanced"
    },
    {
        "id": "UC8ofcOdHNINiPrBA9D59gIQ",
        "name": "Patrick Boyle",
        "domain": "quantitative_finance",
        "default_level": "intermediate"
    },
    {
        "id": "UC9RM-iSvTu1uPJb8XKTjnoA",
        "name": "Wendover Productions",
        "domain": "economics_logistics",
        "default_level": "beginner"
    },
    {
        "id": "UCZ4AMrDcNrfy3X6nsU8-rPg",
        "name": "Economics Explained",
        "domain": "economics_logistics",
        "default_level": "intermediate"
    },
    {
        "id": "UCP5tjEmvPItGyLhmjdwP7Ww",
        "name": "RealLifeLore",
        "domain": "economics_logistics",
        "default_level": "beginner"
    },
    {
        "id": "UCpa-Zb0ZcQjADBjhx5PwJQQ",
        "name": "LegalEagle",
        "domain": "law_legal",
        "default_level": "beginner"
    },

    # --- Creative & Design ---
    {
        "id": "UCOKHwx1QDpZw8iHG-PzQ8TA",
        "name": "Blender Guru",
        "domain": "3d_modeling_vfx",
        "default_level": "beginner"
    },
    {
        "id": "UCj2nPLIW13G6Oce0Jd84F9g",
        "name": "Pwnisher",
        "domain": "3d_modeling_vfx",
        "default_level": "intermediate"
    },
    {
        "id": "UCZHkx_OyRXHb1D3XTqOidRw",
        "name": "The Futur",
        "domain": "graphic_design",
        "default_level": "intermediate"
    },
    {
        "id": "UCoqPRKfOhk9Ez5RHSbGjcbQ",
        "name": "Satori Graphics",
        "domain": "graphic_design",
        "default_level": "beginner"
    },
    {
        "id": "UCQHLxxBFrbfdrk1jF0moTpw",
        "name": "DesignCourse",
        "domain": "ui_ux_design",
        "default_level": "intermediate"
    },
    {
        "id": "UCTjp5aQSaAGSGKPgyST0Lfg",
        "name": "Peter McKinnon",
        "domain": "photography",
        "default_level": "beginner"
    },
    {
        "id": "UCDkJEEIifDzR_2K2p9tnwYQ",
        "name": "Mango Street",
        "domain": "photography",
        "default_level": "intermediate"
    },

    # --- Music & Audio ---
    {
        "id": "UCeZLO2VgbZHeDcongKzzfOw",
        "name": "Adam Neely",
        "domain": "music_theory",
        "default_level": "advanced"
    },
    {
        "id": "UCJquYOG5EL82sKTfH9aMA9Q",
        "name": "Rick Beato",
        "domain": "music_theory",
        "default_level": "intermediate"
    },
    {
        "id": "UCafxR2HWJRmMfSdyZXvZMTw",
        "name": "LOOK MUM NO COMPUTER",
        "domain": "music_theory",
        "default_level": "intermediate"
    },

    # --- Humanities & Personal Development ---
    {
        "id": "UCtYLUTtgS3k1Fg4y5tAhLbw",
        "name": "The School of Life",
        "domain": "philosophy",
        "default_level": "beginner"
    },
    {
        "id": "UC5sx5yxqFQdyQ4Y4VfA6lAQ",
        "name": "Philosophies for Life",
        "domain": "philosophy",
        "default_level": "beginner"
    },
    {
        "id": "UC3lBXcrKM9uY-P7FR1xmJlA",
        "name": "Psych2Go",
        "domain": "psychology",
        "default_level": "beginner"
    },
    {
        "id": "UClHVl2N3jPEbkNJVx-ItQIQ",
        "name": "HealthyGamerGG",
        "domain": "psychology",
        "default_level": "intermediate"
    },
    {
        "id": "UC9tbK5dqwEWdJlunNXaqPPg",
        "name": "History Matters",
        "domain": "history",
        "default_level": "beginner"
    },
    {
        "id": "UCG-KntY7aVnIGXYEBQvmBAQ",
        "name": "Thomas Frank",
        "domain": "productivity",
        "default_level": "beginner"
    },
    {
        "id": "UCU_W0oE_ock8bWKjALiGs8Q",
        "name": "Charisma on Command",
        "domain": "public_speaking",
        "default_level": "beginner"
    },
    {
        "id": "UC26DdVbArFgLNZ1anNBY",
        "name": "Hello Future Me",
        "domain": "writing_creative",
        "default_level": "intermediate"
    },

    # --- Science (Bio/Chem/Astro) ---
    {
        "id": "UCFhXFikryT4aFcLkLw2LBLA",
        "name": "NileRed",
        "domain": "chemistry",
        "default_level": "intermediate"
    },
    {
        "id": "UCX6b17PVsYBQ0ip5gyeme-Q",
        "name": "Khan Academy Biology",
        "domain": "biology",
        "default_level": "beginner"
    },
    {
        "id": "UCciQ8wFcVoIIMi-lfu8-DrQ",
        "name": "Dr. Becky",
        "domain": "astronomy",
        "default_level": "intermediate"
    },
    {
        "id": "UCdJ1UWUwZZU8xsQ8vTIzK7A",
        "name": "Scott Manley",
        "domain": "astronomy",
        "default_level": "advanced"
    },
    {
        "id": "UCsXVk37bltHxD1rDPwtNM8Q",
        "name": "Kurzgesagt",
        "domain": "health_medicine",
        "default_level": "beginner"
    },
    {
        "id": "UCe0TLA0EsQbE-MjuHXevj2A",
        "name": "Jeff Nippard",
        "domain": "fitness_nutrition",
        "default_level": "intermediate"
    },
    {
        "id": "UCX9NJ471o7Wie1DQe94RVIg",
        "name": "Athlean-X",
        "domain": "fitness_nutrition",
        "default_level": "intermediate"
    },

    # --- Other ---
    {
        "id": "UCekQr9znsk2vWxBo3YiLq2w",
        "name": "Binging with Babish",
        "domain": "cooking_culinary",
        "default_level": "beginner"
    },
    {
        "id": "UCzH5n3Ih5kgQoiDAQt2FwLw",
        "name": "Adam Ragusea",
        "domain": "cooking_culinary",
        "default_level": "intermediate"
    },
    {
        "id": "UC_J5bXLsz_t6s0jfjJGw0Hw",
        "name": "English with Lucy",
        "domain": "language_learning_english",
        "default_level": "beginner"
    },
    {
        "id": "UCW5YeuERMmlnqo4oq8vwUpg",
        "name": "Flutter",
        "domain": "mobile_development",
        "default_level": "intermediate"
    },
    {
        "id": "UCYbK_tjZ2OrIZFBvU6CCMiA",
        "name": "Brackeys",
        "domain": "game_development",
        "default_level": "beginner"
    },
    {
        "id": "UCCjyq_K1Xwfg8Lndy1lKpeA",
        "name": "Startup Grind",
        "domain": "entrepreneurship",
        "default_level": "intermediate"
    },
    {
        "id": "UCRhKqR6c30AmgHvz9i4Mh2w",
        "name": "Business Casual",
        "domain": "business_management",
        "default_level": "beginner"
    },
    {
        "id": "UCYHYSq-P5XulFImvq0APMgg",
        "name": "Accounting Stuff",
        "domain": "accounting",
        "default_level": "beginner"
    },
    {
        "id": "UCTJmz2ukP6PZgEPkQi_oG1w",
        "name": "Edspira",
        "domain": "accounting",
        "default_level": "intermediate"
    },
    {
        "id": "UCV_EqXDlZvdIU6Nwxr5W1Ow",
        "name": "Neil Patel",
        "domain": "marketing_digital",
        "default_level": "beginner"
    },
    {
        "id": "UCxSz6JVYmzVhtkraHWZC_0g",
        "name": "Moz",
        "domain": "marketing_digital",
        "default_level": "intermediate"
    },
    {
        "id": "UCWquNQV8Y0_defMKnGKrFOQ",
        "name": "Ahrefs",
        "domain": "seo_content_marketing",
        "default_level": "intermediate"
    }
]

    # Initialize scraper
    scraper = IntegratedYouTubeScraper(youtube_key, openrouter_key)

    # Scrape and enhance
    data = scraper.scrape_and_enhance(channels, videos_per_channel=20)

    # Save results
    output_file = "integrated_output.json"
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2, ensure_ascii=False)

    print(f"\n✓ Saved {len(data['videos'])} videos to {output_file}")
    print(f"  LLM Enhanced: {data['metadata']['llm_enhanced']}")


if __name__ == "__main__":
    main()
