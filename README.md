# Leontine

A modern, responsive client application for audio transcription using the [WhisperX API](https://github.com/jbousquie/whisper_api).

## Project Goals

- Create a user-friendly interface for audio transcription services
- Implement a responsive design that works well on both desktop and mobile devices
- Provide a clean, intuitive UI for uploading audio files and managing transcription settings
- **Display real-time transcription status with queue position and visual progress indicators**
- **Store user preferences for seamless experience across sessions**
- **Persist transcription jobs across browser sessions**
- **Visual processing indicators** for active transcription jobs
- Handle network connectivity issues gracefully with retry mechanisms
- Implement proper error handling and user feedback

## Features

- **File Upload**: Drag-and-drop or file selection interface for audio files
- **Status Tracking**: Real-time updates on transcription progress, queue position, and API queue state
- **Result Management**: Download transcription results as text files
- **Responsive Design**: Works on desktop, tablet, and mobile devices
- **Persistent Settings**: Save user preferences between sessions
- **Job Persistence**: Resume transcription jobs after browser restart or page reload
- **Enhanced Visual Indicators**: Animated dots for status updates
- **Clear Notifications**: Contextual feedback for user actions


## Usage

1. Set your API URL in the settings section
2. The application will check the API status and display server queue information
3. Upload an audio file by dragging and dropping or using the file selector
4. Click "Transcribe Audio" and monitor the progress
5. When complete, download your transcription as a text file
6. If you close the browser during transcription, the job will automatically resume when you return to the application
7. If a job fails or is not found, the application will notify the user and propose him to submit a new transcription.
## Development

Leontine is built with [Dioxus](https://dioxuslabs.com/) and Rust.

## WhisperX API

Leontine is designed to work with the WhisperX API, which provides powerful audio transcription services with features like:

- Multi-language support
- Speaker diarization
- Various output formats
- Queueing system for handling multiple requests
- Status endpoint for monitoring API health and queue state

For more information about the API, see the [WhisperX API repository](https://github.com/jbousquie/whisper_api).

## License

[MIT License](LICENSE)
