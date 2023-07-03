module Remote where

import Prelude hiding (div)

import Control.Monad.Rec.Class (forever)
import Data.Argonaut.Encode (toJsonString)
import Data.Array (length, mapMaybe, nub, null, sort, take, uncons)
import Data.Either (Either(..))
import Data.HTTP.Method (Method(..))
import Data.Int (floor)
import Data.Maybe (Maybe(..), fromMaybe)
import Data.Newtype (wrap)
import Data.String (drop, joinWith, lastIndexOf, split, stripPrefix)
import Data.Tuple.Nested ((/\))
import Effect (Effect)
import Effect.Aff (Aff, delay, launchAff_)
import Effect.Class (liftEffect)
import Fetch (fetch)
import Fetch.Argonaut.Json (fromJson)
import Instruction (Behaviour(..), Instruction(..))
import Muon (Html, Muon, Signal, a, button, click, div, i, muon, on, state, text, (:=))
import PlayerState (PlayerState)

type State = {
  files :: Array String,
  prefix :: Array String,
  playerState :: Maybe PlayerState
}

type StateChans = {
  files :: Array String -> Effect Unit,
  prefix :: Array String -> Effect Unit,
  playerState :: Maybe PlayerState -> Effect Unit
}

main :: Effect Unit
main = muon =<< app

app :: Effect (Signal (Muon Html))
app = do
  sig /\ chans <- state { files: [], prefix: [], playerState: Nothing }
  list chans
  poll chans
  pure $ sig <#> \{ files, prefix, playerState } -> pure $
    div ["class" := "container pt-5"] [
      div ["class" := "row"] [
        div ["class" := "col-12"] $
          case playerState of
            Just { path, duration, time } ->
              [
                div ["class" := "card"] [
                  div ["class" := "card-body text-center"] [
                    div ["class" := "h3 m-0 text-warning"] [text $ fileName path],
                    div ["class" := "my-4"] [
                      button ["class" := "btn btn-primary m-2", on click (const $ play path $ pure $ max 0.0 (time - 60.0))] [
                        i ["class" := "h1 m-0 bx bx-chevrons-left"] []
                      ],
                      button ["class" := "btn btn-primary m-2", on click (const $ resume path)] [
                        i ["class" := "h1 m-0 bx bx-play"] []
                      ],
                      button ["class" := "btn btn-primary m-2", on click (const $ pause path)] [
                        i ["class" := "h1 m-0 bx bx-pause"] []
                      ],
                      button ["class" := "btn btn-primary m-2", on click (const stop)] [
                        i ["class" := "h1 m-0 bx bx-stop"] []
                      ],
                      button ["class" := "btn btn-primary", on click (const $ play path $ pure $ min duration (time + 60.0))] [
                        i ["class" := "h1 m-0 bx bx-chevrons-right"] []
                      ]
                    ],
                    div ["class" := "progress"] [
                      div
                        [
                          "class" := "progress-bar",
                          "role" := "progressbar",
                          "style" := ("width: " <> show (floor $ (time / duration) * 100.0) <> "%;")
                        ]
                        []
                    ]
                  ]
                ]
              ]
            Nothing ->
              [
                div ["class" := "card"] [
                  div ["class" := "card-body"] [
                    ifHtml (not null prefix) $
                      div ["class" := "mb-3 p-3"] [
                        a ["href" := "#", on click (const $ chans.prefix $ parentDir prefix)] [
                          i ["class" := "bx bx-arrow-back mr-2"] [],
                          text (joinWith "/" prefix)
                        ]
                      ],
                    div [] $
                      getFileList prefix files <#> case _ of
                        Left { name, path } ->
                          div ["class" := "mb-3 p-3 border"] [
                            a ["href" := "#", on click (const $ play path Nothing)] [text name]
                          ]
                        Right dir ->
                          div ["class" := "mb-3 p-3 border"] [
                            a ["href" := "#", on click (const $ chans.prefix $ childDir prefix dir)] [text dir]
                          ]
                  ]
                ]
              ]
      ]
    ]

ifHtml :: Boolean -> Html -> Html
ifHtml c h = if c then h else text ""

getFileList :: Array String -> Array String -> Array (Either { name :: String, path :: String } String)
getFileList prefix = sort <<< nub <<< mapMaybe toDesc
  where
  toDesc path = do
    suffix <- uncons <<< split (wrap "/") =<< stripPrefix (wrap $ prefixPath prefix) path
    pure case uncons suffix.tail of
      Nothing ->
        Left { name: suffix.head, path }
      Just _ ->
        Right suffix.head

prefixPath :: Array String -> String
prefixPath = case _ of
  [] ->
    ""
  path ->
    joinWith "/" path <> "/"

childDir :: Array String -> String -> Array String
childDir current child = current <> [child]

parentDir :: Array String -> Array String
parentDir dir = take (length dir - 1) dir

fileName :: String -> String
fileName path = fromMaybe path do
  ix <- lastIndexOf (wrap "/") path
  pure $ drop (ix + 1) path

--
-- Network stuff
--

list :: StateChans -> Effect Unit
list chans = launchAff_ $ forever do
  listFiles chans
  delay (wrap 5000.0)

listFiles :: StateChans -> Aff Unit
listFiles chans = do
  { json } <- fetch "/ls" {
    method: GET,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
  files <- fromJson json
  liftEffect $ chans.files files

poll :: StateChans -> Effect Unit
poll chans = launchAff_ $ forever do
  pollPlayerState chans
  delay (wrap 250.0)

pollPlayerState :: StateChans -> Aff Unit
pollPlayerState chans = do
  { json } <- fetch "/state" {
    method: GET,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
  liftEffect <<< chans.playerState =<< fromJson json

play :: String -> Maybe Number -> Effect Unit
play path from = instruction (Play path from Playing)

resume :: String -> Effect Unit
resume path = instruction (Play path Nothing Playing)

pause :: String -> Effect Unit
pause path = instruction (Play path Nothing Paused)

stop :: Effect Unit
stop = instruction Idle

instruction :: Instruction -> Effect Unit
instruction i = launchAff_ do
  void $ fetch "/instruction" {
    method: POST,
    body: toJsonString i,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
