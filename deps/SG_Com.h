#pragma once

#ifdef __cplusplus
#define SG_EXTERN extern "C"
#else
#define SG_EXTERN
#endif

#if defined(_WIN32)
    #ifdef BUILDING_DLL
    #define SG_EXPORT SG_EXTERN __declspec(dllexport)
    #else
    #define SG_EXPORT SG_EXTERN __declspec(dllimport)
    #endif
#else
    #define SG_EXPORT SG_EXTERN __attribute__((visibility("default")))
#endif

#include <stddef.h>
#include <stdint.h>

typedef enum {
    SG_ERROR_OK, /// No error
    SG_ERROR_BUFFER_OVERFLOW, /// Buffer is too short
    SG_ERROR_UNKNOWN0,
    SG_ERROR_LICENSE_INIT, /// Error initializing licensing
    SG_ERROR_UNKNOWN1,
    SG_ERROR_LICENSE_CHECKOUT, /// Error checking out license
    SG_ERROR_UNKNOWN2,
    SG_ERROR_INVALID_TRANSCEIVER, /// Invalid TransceiverPtr
    SG_ERROR_INVALID_INPUT_TRAITS, /// Invalid input traits
    SG_ERROR_INPUT_FAILURE, /// Input failure
    SG_ERROR_INVALID_OUTPUT_TRAITS, /// Invalid output traits
    SG_ERROR_INVALID_USER_ID, /// Invalid user ID
    SG_ERROR_INVALID_ANIMATION_NODE, /// Invalid animation node
    SG_ERROR_INVALID_ANIMATION_CHANNEL, /// Invalid animation channel
} SG_Error;

typedef enum {
    SG_LOG_NONE, /// No logging enabled
    SG_LOG_ERROR, /// Only errors will be logged
    SG_LOG_ALL /// All log messages will be printed
} SG_LogLevel;

typedef enum {
    SG_RATE_8KHZ, /// 8kHz
    SG_RATE_12KHZ, /// 12kHz
    SG_RATE_16KHZ, /// 16kHz
    SG_RATE_24KHZ, /// 24kHz
    SG_RATE_32KHZ, /// 32kHz
    SG_RATE_48KHZ /// 48kHz
} SG_SampleRate;

typedef enum {
    SG_SAMPLE_PCM8, /// PCM8
    SG_SAMPLE_PCM16, /// PCM16
    SG_SAMPLE_PCM32, /// PCM32
    SG_SAMPLE_FLOAT32, /// Float32
    SG_SAMPLE_FLOAT64 /// Float64
} SG_SampleType;

typedef enum {
    SG_ANIM_DEFORMER,
    SG_ANIM_CONTROL
} SG_AnimationType;

typedef enum {
    SG_NODE_JOINT,   /// Joint
    SG_NODE_BLEND_SHAPE, /// Blend Shape  
    SG_NODE_CONTROL /// Control
} SG_AnimationNodeType;

typedef enum {
    SG_OUTPUT_NONE = 0,
    SG_OUTPUT_ANIMATION = 2, /// Output animation/morphs
    SG_OUTPUT_AUDIO = 4, /// Output audio
    SG_OUTPUT_USER_DEFINED = 8, /// Output user data
} SG_OutputDataType;

typedef struct SG_Transceiver* SG_TransceiverPtr;

typedef void (*SG_OnTransmit)(
	uint8_t* packet,
	size_t size
);
	
typedef struct SG_InputTraits{
    SG_SampleType sample_type;
    SG_SampleRate sample_rate;
    size_t user_sample_size;
} SG_InputTraits;

typedef struct SG_OutputTraits{
    SG_OutputDataType output_type; /// Flags
    uint32_t anim_node_count;
    SG_SampleType sample_type;
    uint32_t sample_rate;
    uint32_t user_sample_size;
    uint32_t user_sample_rate;
} SG_OutputTraits;

typedef struct SG_AnimationNodeInfo{
    char name[1024];
    SG_AnimationNodeType type;
    uint32_t channel_count;
} SG_AnimationNodeInfo;

SG_EXPORT SG_Error SG_Initialize( void );

SG_EXPORT SG_Error SG_Shutdown( void );

SG_EXPORT const char* SG_GetVersionString( void );

SG_EXPORT uint32_t SG_GetVersionNumber( void );

SG_EXPORT SG_Error SG_SetLoggingLevel( SG_LogLevel level );

SG_EXPORT SG_Error SG_STDLN_CreateTransceiver(
	uint8_t* algorithm_data,
	size_t algorithms_size,
	uint8_t* character_data,
	size_t character_size,
	SG_InputTraits *input_traits,
	SG_OutputDataType output_type,
	SG_OnTransmit on_transmit, // can be null
	SG_AnimationType anim_type,
	uint32_t unk,
	float input_size_seconds,
	float playback_size_ms,
	SG_TransceiverPtr *transceiver
);
	
SG_EXPORT SG_Error SG_STDLN_DestroyTransceiver(
    SG_TransceiverPtr transceiver
);

SG_EXPORT SG_Error SG_STDLN_ConnectUser(
    SG_TransceiverPtr transceiver,
	uint64_t user_ID,
	uint8_t* config,
	size_t config_size
);

SG_EXPORT SG_Error SG_STDLN_DisconnectUser(
    SG_TransceiverPtr transceiver,
	uint64_t user_ID
);
	
SG_EXPORT SG_Error SG_STDLN_GetDecodingConfiguration(
    SG_TransceiverPtr transceiver,
	uint8_t* *config,
	size_t *config_size
);

SG_EXPORT SG_Error SG_STDLN_Receive(
    SG_TransceiverPtr transceiver,
	uint64_t user_ID,
	uint8_t* packet,
	size_t packet_size
);

/// Does nothing?
SG_EXPORT SG_Error SG_STDLN_SynchronizeClock(
    SG_TransceiverPtr transceiver,
	const char* current_time
);

SG_EXPORT SG_Error SG_GetAnimationNodeInfo(
    SG_TransceiverPtr transceiver,
    uint64_t user_ID,
    uint32_t node_index,
    SG_AnimationNodeInfo *node_info
);

SG_EXPORT SG_Error SG_GetAnimationChannelName(
    SG_TransceiverPtr transceiver,
    uint64_t user_ID,
    const char* node_name,
    uint32_t channel_idx,
    char *channel_name, 
    size_t channel_name_size
);

SG_EXPORT SG_Error SG_UpdateInputTraits(
	SG_InputTraits* input_traits,
	SG_TransceiverPtr transceiver
);

SG_EXPORT SG_Error SG_GetOutputTraits(
    SG_TransceiverPtr transceiver, 
    uint64_t user_ID,
    SG_OutputTraits *output_traits
);

SG_EXPORT SG_Error SG_Input(
    SG_TransceiverPtr transceiver,
    uint8_t* audio_data,
    uint32_t sample_count,
    uint8_t* user_data
);

SG_EXPORT SG_Error SG_AdvanceOutput(
    SG_TransceiverPtr transceiver, 
    float delta_ms
);

SG_EXPORT SG_Error SG_GetOutputAnimation(
    SG_TransceiverPtr transceiver,
    uint64_t user_ID,
    const char* node_name,
    float* *animation
);

SG_EXPORT SG_Error SG_GetOutputAudio(
    SG_TransceiverPtr transceiver,
    uint64_t user_ID,
    uint8_t* *audio,
    uint32_t *sample_count
);
   
SG_EXPORT SG_Error SG_GetOutputUserData(
    SG_TransceiverPtr transceiver,
    uint64_t user_ID,
    uint8_t* *user_data,
    uint32_t *num_samples
);

SG_EXPORT SG_Error SG_GetMoodList(
    SG_TransceiverPtr transceiver,
    char* mood_list,
    size_t mood_list_size
);

SG_EXPORT SG_Error SG_GetCurrentMood(
    SG_TransceiverPtr transceiver,
    char* mood,
    size_t mood_size
);
	
SG_EXPORT SG_Error SG_SetMood(
    SG_TransceiverPtr transceiver,
    const char* mood
);

SG_EXPORT SG_Error SG_GetCurrentIntensity(
    SG_TransceiverPtr transceiver,
    float *intensity
);
	
SG_EXPORT SG_Error SG_SetIntensity(
    SG_TransceiverPtr transceiver,
    float intensity
);
