using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Newtonsoft.Json;
using UnityEditor;
using UnityEngine;

namespace Builds
{
    public class BuildSystem
    {
        static string GetTelebuildLocal()
        {
            return Environment.GetCommandLineArgs().SkipWhile(item => !item.Equals("-teleroot")).Skip(1).First();
        }

        static string GetBuildPathLocal(BuildSettings settings)
        {
            return settings.build_path;
        }

        static string GetTelebuildRoot()
        {
            return Path.Combine(Directory.GetParent(Application.dataPath).FullName, GetTelebuildLocal());
        }

        static string[] GetScenes()
        {
            List<string> scenes = new List<string>();
            for (int i = 0; i < EditorBuildSettings.scenes.Length; i++)
            {
                if (EditorBuildSettings.scenes[i].enabled)
                {
                    scenes.Add(EditorBuildSettings.scenes[i].path);
                }
            }

            return scenes.ToArray();
        }

        static void Build()
        {
            var telebuildRoot = GetTelebuildRoot();
            var telebuildLocal = GetTelebuildLocal();
            
            var settings = LoadSettings(telebuildRoot);
            SetKeystorePassword(settings);
            var buildParams = new BuildParams(settings, GetScenes(), telebuildRoot, telebuildLocal,
                GetBuildPathLocal(settings));
            
            UnityOutput output;
            switch (settings.platform)
            {
                case BuildPlatform.AndroidDevelopment:
                    output = AndroidBuild.AndroidDevelopment(buildParams);
                    break;
                case BuildPlatform.AndroidRelease:
                    output = AndroidBuild.AndroidRelease(buildParams);
                    break;
                default:
                    throw new ArgumentOutOfRangeException();
            }

            WriteResult(telebuildRoot, output);
            Application.Quit(output.exit_code);
        }

        static void SetKeystorePassword(BuildSettings settings)
        {
            PlayerSettings.keyaliasPass = settings.keystore_password;
            PlayerSettings.keystorePass = settings.keystore_password;
        }

        static void WriteResult(string telebuildRoot, UnityOutput output)
        {
            string jsonString = JsonConvert.SerializeObject(output);
            File.WriteAllText(Path.Combine(telebuildRoot, "output.json"), jsonString);
        }

        static BuildSettings LoadSettings(string telebuildRoot)
        {
            var path = Path.Combine(telebuildRoot, "settings.json");
            return JsonConvert.DeserializeObject<BuildSettings>(File.ReadAllText(path));
        }
    }
}
