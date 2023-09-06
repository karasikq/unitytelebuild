using System;
using System.IO;
using Newtonsoft.Json;
using Newtonsoft.Json.Converters;

namespace Builds
{
    [JsonConverter(typeof(StringEnumConverter))]
    public enum BuildPlatform
    {
        AndroidDevelopment,
        AndroidRelease,
    }

    [Serializable]
    public class UnityOutput
    {
        public BuildPlatform platform;
        public string build_path;
        public int exit_code;
    }

    [Serializable]
    public class BuildSettings
    {
        public BuildPlatform platform;
        public string keystore_password;
        public string build_path;
    }

    public class BuildParams
    {
        public BuildSettings Settings;
        public string[] Scenes;
        public string TelebuildPath;
        public string TelebuildPathLocal;
        public string BuildPathLocal;
        public string BuildPath;

        public BuildParams(BuildSettings settings, string[] scenes, string telebuildPath, string telebuildPathLocal, string buildPathLocal)
        {
            Settings = settings;
            Scenes = scenes;
            TelebuildPath = telebuildPath;
            TelebuildPathLocal = telebuildPathLocal;
            BuildPathLocal = buildPathLocal;
            BuildPath = Path.Combine(TelebuildPath, settings.build_path);
        }
    }
}