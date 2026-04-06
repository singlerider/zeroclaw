//! Channel factory — instantiation and routing of configured channels.
//!
//! Contains `build_channel_by_id` (for one-shot channel sends),
//! `collect_configured_channels` (for runtime startup), and related helpers.

use anyhow::{Context, Result};
use std::sync::Arc;

use super::traits::{Channel, SendMessage};
use crate::config::Config;

// Conditionally import channel types
#[cfg(feature = "channel-bluesky")]
use super::BlueskyChannel;
#[cfg(feature = "channel-clawdtalk")]
use super::clawdtalk::ClawdTalkChannel;
#[cfg(feature = "channel-dingtalk")]
use super::DingTalkChannel;
#[cfg(feature = "channel-discord")]
use super::DiscordChannel;
#[cfg(feature = "channel-discord")]
use super::DiscordHistoryChannel;
#[cfg(feature = "channel-email")]
use super::EmailChannel;
#[cfg(feature = "channel-email")]
use super::GmailPushChannel;
#[cfg(feature = "channel-imessage")]
use super::IMessageChannel;
#[cfg(feature = "channel-irc")]
use super::irc;
#[cfg(feature = "channel-irc")]
use super::IrcChannel;
#[cfg(feature = "channel-lark")]
use super::LarkChannel;
#[cfg(feature = "channel-linq")]
use super::LinqChannel;
#[cfg(feature = "channel-matrix")]
use super::MatrixChannel;
#[cfg(feature = "channel-mattermost")]
use super::MattermostChannel;
#[cfg(feature = "channel-mochat")]
use super::MochatChannel;
#[cfg(feature = "channel-nextcloud")]
use super::NextcloudTalkChannel;
#[cfg(feature = "channel-notion")]
use super::NotionChannel;
#[cfg(feature = "channel-qq")]
use super::QQChannel;
#[cfg(feature = "channel-reddit")]
use super::RedditChannel;
#[cfg(feature = "channel-signal")]
use super::SignalChannel;
#[cfg(feature = "channel-slack")]
use super::SlackChannel;
#[cfg(feature = "channel-telegram")]
use super::TelegramChannel;
#[cfg(feature = "channel-twitter")]
use super::TwitterChannel;
#[cfg(feature = "voice-wake")]
use super::VoiceWakeChannel;
#[cfg(feature = "channel-wati")]
use super::WatiChannel;
#[cfg(feature = "channel-webhook")]
use super::WebhookChannel;
#[cfg(feature = "channel-wecom")]
use super::WeComChannel;
#[cfg(feature = "channel-whatsapp-cloud")]
use super::WhatsAppChannel;
#[cfg(feature = "whatsapp-web")]
use super::whatsapp_web::WhatsAppWebChannel;

pub(super) fn build_channel_by_id(config: &Config, channel_id: &str) -> Result<Arc<dyn Channel>> {
    match channel_id {
        "telegram" => {
            #[cfg(feature = "channel-telegram")]
            {
                let tg = config
                    .channels_config
                    .telegram
                    .as_ref()
                    .context("Telegram channel is not configured")?;
                let ack = tg
                    .ack_reactions
                    .unwrap_or(config.channels_config.ack_reactions);
                Ok(Arc::new(
                    TelegramChannel::new(
                        tg.bot_token.clone(),
                        tg.allowed_users.clone(),
                        tg.mention_only,
                    )
                    .with_ack_reactions(ack)
                    .with_streaming(tg.stream_mode, tg.draft_update_interval_ms)
                    .with_transcription(config.transcription.clone())
                    .with_tts(config.tts.clone())
                    .with_workspace_dir(config.workspace_dir.clone()),
                ))
            }
            #[cfg(not(feature = "channel-telegram"))]
            {
                anyhow::bail!("Telegram channel requires the `channel-telegram` feature");
            }
        }
        "discord" => {
            #[cfg(feature = "channel-discord")]
            {
                let dc = config
                    .channels_config
                    .discord
                    .as_ref()
                    .context("Discord channel is not configured")?;
                Ok(Arc::new(
                    DiscordChannel::new(
                        dc.bot_token.clone(),
                        dc.guild_id.clone(),
                        dc.allowed_users.clone(),
                        dc.listen_to_bots,
                        dc.mention_only,
                    )
                    .with_streaming(
                        dc.stream_mode,
                        dc.draft_update_interval_ms,
                        dc.multi_message_delay_ms,
                    )
                    .with_transcription(config.transcription.clone())
                    .with_stall_timeout(dc.stall_timeout_secs),
                ))
            }
            #[cfg(not(feature = "channel-discord"))]
            anyhow::bail!("Discord channel requires the `channel-discord` feature")
        }
        "slack" => {
            #[cfg(feature = "channel-slack")]
            {
                let sl = config
                    .channels_config
                    .slack
                    .as_ref()
                    .context("Slack channel is not configured")?;
                Ok(Arc::new(
                    SlackChannel::new(
                        sl.bot_token.clone(),
                        sl.app_token.clone(),
                        sl.channel_id.clone(),
                        sl.channel_ids.clone(),
                        sl.allowed_users.clone(),
                    )
                    .with_workspace_dir(config.workspace_dir.clone())
                    .with_markdown_blocks(sl.use_markdown_blocks)
                    .with_transcription(config.transcription.clone())
                    .with_streaming(sl.stream_drafts, sl.draft_update_interval_ms)
                    .with_cancel_reaction(sl.cancel_reaction.clone()),
                ))
            }
            #[cfg(not(feature = "channel-slack"))]
            anyhow::bail!("Slack channel requires the `channel-slack` feature")
        }
        "mattermost" => {
            #[cfg(feature = "channel-mattermost")]
            {
                let mm = config
                    .channels_config
                    .mattermost
                    .as_ref()
                    .context("Mattermost channel is not configured")?;
                Ok(Arc::new(MattermostChannel::new(
                    mm.url.clone(),
                    mm.bot_token.clone(),
                    mm.channel_id.clone(),
                    mm.allowed_users.clone(),
                    mm.thread_replies.unwrap_or(true),
                    mm.mention_only.unwrap_or(false),
                )))
            }
            #[cfg(not(feature = "channel-mattermost"))]
            anyhow::bail!("Mattermost channel requires the `channel-mattermost` feature")
        }
        "signal" => {
            #[cfg(feature = "channel-signal")]
            {
                let sg = config
                    .channels_config
                    .signal
                    .as_ref()
                    .context("Signal channel is not configured")?;
                Ok(Arc::new(SignalChannel::new(
                    sg.http_url.clone(),
                    sg.account.clone(),
                    sg.group_id.clone(),
                    sg.allowed_from.clone(),
                    sg.ignore_attachments,
                    sg.ignore_stories,
                )))
            }
            #[cfg(not(feature = "channel-signal"))]
            anyhow::bail!("Signal channel requires the `channel-signal` feature")
        }
        "matrix" => {
            #[cfg(feature = "channel-matrix")]
            {
                let mx = config
                    .channels_config
                    .matrix
                    .as_ref()
                    .context("Matrix channel is not configured")?;
                Ok(Arc::new(MatrixChannel::new(
                    mx.homeserver.clone(),
                    mx.access_token.clone(),
                    mx.room_id.clone(),
                    mx.allowed_users.clone(),
                )))
            }
            #[cfg(not(feature = "channel-matrix"))]
            {
                anyhow::bail!("Matrix channel requires the `channel-matrix` feature");
            }
        }
        "whatsapp" | "whatsapp-web" | "whatsapp_web" => {
            #[cfg(feature = "whatsapp-web")]
            {
                let wa = config
                    .channels_config
                    .whatsapp
                    .as_ref()
                    .context("WhatsApp channel is not configured")?;
                if !wa.is_web_config() {
                    anyhow::bail!(
                        "WhatsApp channel send requires Web mode (session_path must be set)"
                    );
                }
                Ok(Arc::new(WhatsAppWebChannel::new(
                    wa.session_path.clone().unwrap_or_default(),
                    wa.pair_phone.clone(),
                    wa.pair_code.clone(),
                    wa.allowed_numbers.clone(),
                    wa.mention_only,
                    wa.mode.clone(),
                    wa.dm_policy.clone(),
                    wa.group_policy.clone(),
                    wa.self_chat_mode,
                )))
            }
            #[cfg(not(feature = "whatsapp-web"))]
            {
                anyhow::bail!("WhatsApp channel requires the `whatsapp-web` feature");
            }
        }
        "qq" => {
            #[cfg(feature = "channel-qq")]
            {
                let qq = config
                    .channels_config
                    .qq
                    .as_ref()
                    .context("QQ channel is not configured")?;
                Ok(Arc::new(QQChannel::new(
                    qq.app_id.clone(),
                    qq.app_secret.clone(),
                    qq.allowed_users.clone(),
                )))
            }
            #[cfg(not(feature = "channel-qq"))]
            anyhow::bail!("QQ channel requires the `channel-qq` feature")
        }
        other => anyhow::bail!(
            "Unknown channel '{other}'. Supported: telegram, discord, slack, mattermost, signal, matrix, whatsapp, qq"
        ),
    }
}

/// Send a one-off message to a configured channel.
pub(super) async fn send_channel_message(
    config: &Config,
    channel_id: &str,
    recipient: &str,
    message: &str,
) -> Result<()> {
    let channel = build_channel_by_id(config, channel_id)?;
    let msg = SendMessage::new(message, recipient);
    channel
        .send(&msg)
        .await
        .with_context(|| format!("Failed to send message via {channel_id}"))?;
    println!("Message sent via {channel_id}.");
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChannelHealthState {
    Healthy,
    Unhealthy,
    Timeout,
}

pub(super) fn classify_health_result(
    result: &std::result::Result<bool, tokio::time::error::Elapsed>,
) -> ChannelHealthState {
    match result {
        Ok(true) => ChannelHealthState::Healthy,
        Ok(false) => ChannelHealthState::Unhealthy,
        Err(_) => ChannelHealthState::Timeout,
    }
}

pub(super) struct ConfiguredChannel {
    pub(super) display_name: &'static str,
    pub(super) channel: Arc<dyn Channel>,
}

pub(super) fn collect_configured_channels(
    config: &Config,
    matrix_skip_context: &str,
) -> Vec<ConfiguredChannel> {
    let _ = matrix_skip_context;
    let mut channels = Vec::new();

    #[cfg(feature = "channel-telegram")]
    if let Some(ref tg) = config.channels_config.telegram {
        let ack = tg
            .ack_reactions
            .unwrap_or(config.channels_config.ack_reactions);
        channels.push(ConfiguredChannel {
            display_name: "Telegram",
            channel: Arc::new(
                TelegramChannel::new(
                    tg.bot_token.clone(),
                    tg.allowed_users.clone(),
                    tg.mention_only,
                )
                .with_ack_reactions(ack)
                .with_streaming(tg.stream_mode, tg.draft_update_interval_ms)
                .with_transcription(config.transcription.clone())
                .with_tts(config.tts.clone())
                .with_workspace_dir(config.workspace_dir.clone())
                .with_proxy_url(tg.proxy_url.clone()),
            ),
        });
    }

    #[cfg(not(feature = "channel-telegram"))]
    if config.channels_config.telegram.is_some() {
        tracing::warn!(
            "Telegram channel is configured but this build was compiled without `channel-telegram`; skipping."
        );
    }

    #[cfg(feature = "channel-discord")]
    if let Some(ref dc) = config.channels_config.discord {
        channels.push(ConfiguredChannel {
            display_name: "Discord",
            channel: Arc::new(
                DiscordChannel::new(
                    dc.bot_token.clone(),
                    dc.guild_id.clone(),
                    dc.allowed_users.clone(),
                    dc.listen_to_bots,
                    dc.mention_only,
                )
                .with_streaming(
                    dc.stream_mode,
                    dc.draft_update_interval_ms,
                    dc.multi_message_delay_ms,
                )
                .with_proxy_url(dc.proxy_url.clone())
                .with_transcription(config.transcription.clone())
                .with_stall_timeout(dc.stall_timeout_secs),
            ),
        });
    }

    #[cfg(feature = "channel-discord")]
    if let Some(ref dh) = config.channels_config.discord_history {
        match crate::memory::SqliteMemory::new_named(&config.workspace_dir, "discord") {
            Ok(discord_mem) => {
                channels.push(ConfiguredChannel {
                    display_name: "Discord History",
                    channel: Arc::new(
                        DiscordHistoryChannel::new(
                            dh.bot_token.clone(),
                            dh.guild_id.clone(),
                            dh.allowed_users.clone(),
                            dh.channel_ids.clone(),
                            Arc::new(discord_mem),
                            dh.store_dms,
                            dh.respond_to_dms,
                        )
                        .with_proxy_url(dh.proxy_url.clone()),
                    ),
                });
            }
            Err(e) => {
                tracing::error!("discord_history: failed to open discord.db: {e}");
            }
        }
    }

    #[cfg(feature = "channel-slack")]
    if let Some(ref sl) = config.channels_config.slack {
        channels.push(ConfiguredChannel {
            display_name: "Slack",
            channel: Arc::new(
                SlackChannel::new(
                    sl.bot_token.clone(),
                    sl.app_token.clone(),
                    sl.channel_id.clone(),
                    sl.channel_ids.clone(),
                    sl.allowed_users.clone(),
                )
                .with_thread_replies(sl.thread_replies.unwrap_or(true))
                .with_group_reply_policy(sl.mention_only, Vec::new())
                .with_workspace_dir(config.workspace_dir.clone())
                .with_markdown_blocks(sl.use_markdown_blocks)
                .with_proxy_url(sl.proxy_url.clone())
                .with_transcription(config.transcription.clone())
                .with_streaming(sl.stream_drafts, sl.draft_update_interval_ms)
                .with_cancel_reaction(sl.cancel_reaction.clone()),
            ),
        });
    }

    #[cfg(feature = "channel-mattermost")]
    if let Some(ref mm) = config.channels_config.mattermost {
        channels.push(ConfiguredChannel {
            display_name: "Mattermost",
            channel: Arc::new(
                MattermostChannel::new(
                    mm.url.clone(),
                    mm.bot_token.clone(),
                    mm.channel_id.clone(),
                    mm.allowed_users.clone(),
                    mm.thread_replies.unwrap_or(true),
                    mm.mention_only.unwrap_or(false),
                )
                .with_proxy_url(mm.proxy_url.clone())
                .with_transcription(config.transcription.clone()),
            ),
        });
    }

    #[cfg(feature = "channel-imessage")]
    if let Some(ref im) = config.channels_config.imessage {
        channels.push(ConfiguredChannel {
            display_name: "iMessage",
            channel: Arc::new(IMessageChannel::new(im.allowed_contacts.clone())),
        });
    }

    #[cfg(feature = "channel-matrix")]
    if let Some(ref mx) = config.channels_config.matrix {
        channels.push(ConfiguredChannel {
            display_name: "Matrix",
            channel: Arc::new(
                MatrixChannel::new_full(
                    mx.homeserver.clone(),
                    mx.access_token.clone(),
                    mx.room_id.clone(),
                    mx.allowed_users.clone(),
                    mx.allowed_rooms.clone(),
                    mx.user_id.clone(),
                    mx.device_id.clone(),
                    config.config_path.parent().map(|path| path.to_path_buf()),
                    mx.recovery_key.clone(),
                )
                .with_streaming(
                    mx.stream_mode,
                    mx.draft_update_interval_ms,
                    mx.multi_message_delay_ms,
                )
                .with_transcription(config.transcription.clone()),
            ),
        });
    }

    #[cfg(not(feature = "channel-matrix"))]
    if config.channels_config.matrix.is_some() {
        tracing::warn!(
            "Matrix channel is configured but this build was compiled without `channel-matrix`; skipping Matrix {}.",
            matrix_skip_context
        );
    }

    #[cfg(feature = "channel-signal")]
    if let Some(ref sig) = config.channels_config.signal {
        channels.push(ConfiguredChannel {
            display_name: "Signal",
            channel: Arc::new(
                SignalChannel::new(
                    sig.http_url.clone(),
                    sig.account.clone(),
                    sig.group_id.clone(),
                    sig.allowed_from.clone(),
                    sig.ignore_attachments,
                    sig.ignore_stories,
                )
                .with_proxy_url(sig.proxy_url.clone()),
            ),
        });
    }

    #[cfg(feature = "channel-whatsapp-cloud")]
    if let Some(ref wa) = config.channels_config.whatsapp {
        if wa.is_ambiguous_config() {
            tracing::warn!(
                "WhatsApp config has both phone_number_id and session_path set; preferring Cloud API mode. Remove one selector to avoid ambiguity."
            );
        }
        // Runtime negotiation: detect backend type from config
        match wa.backend_type() {
            "cloud" => {
                // Cloud API mode: requires phone_number_id, access_token, verify_token
                if wa.is_cloud_config() {
                    channels.push(ConfiguredChannel {
                        display_name: "WhatsApp",
                        channel: Arc::new(
                            WhatsAppChannel::new(
                                wa.access_token.clone().unwrap_or_default(),
                                wa.phone_number_id.clone().unwrap_or_default(),
                                wa.verify_token.clone().unwrap_or_default(),
                                wa.allowed_numbers.clone(),
                            )
                            .with_proxy_url(wa.proxy_url.clone())
                            .with_dm_mention_patterns(wa.dm_mention_patterns.clone())
                            .with_group_mention_patterns(wa.group_mention_patterns.clone()),
                        ),
                    });
                } else {
                    tracing::warn!(
                        "WhatsApp Cloud API configured but missing required fields (phone_number_id, access_token, verify_token)"
                    );
                }
            }
            "web" => {
                // Web mode: requires session_path
                #[cfg(feature = "whatsapp-web")]
                if wa.is_web_config() {
                    channels.push(ConfiguredChannel {
                        display_name: "WhatsApp",
                        channel: Arc::new(
                            WhatsAppWebChannel::new(
                                wa.session_path.clone().unwrap_or_default(),
                                wa.pair_phone.clone(),
                                wa.pair_code.clone(),
                                wa.allowed_numbers.clone(),
                                wa.mention_only,
                                wa.mode.clone(),
                                wa.dm_policy.clone(),
                                wa.group_policy.clone(),
                                wa.self_chat_mode,
                            )
                            .with_transcription(config.transcription.clone())
                            .with_tts(config.tts.clone())
                            .with_dm_mention_patterns(wa.dm_mention_patterns.clone())
                            .with_group_mention_patterns(wa.group_mention_patterns.clone()),
                        ),
                    });
                } else {
                    tracing::warn!("WhatsApp Web configured but session_path not set");
                }
                #[cfg(not(feature = "whatsapp-web"))]
                {
                    tracing::warn!(
                        "WhatsApp Web backend requires 'whatsapp-web' feature. Enable with: cargo build --features whatsapp-web"
                    );
                    eprintln!(
                        "  ⚠ WhatsApp Web is configured but the 'whatsapp-web' feature is not compiled in."
                    );
                    eprintln!("    Rebuild with: cargo build --features whatsapp-web");
                }
            }
            _ => {
                tracing::warn!(
                    "WhatsApp config invalid: neither phone_number_id (Cloud API) nor session_path (Web) is set"
                );
            }
        }
    }

    #[cfg(feature = "channel-linq")]
    if let Some(ref lq) = config.channels_config.linq {
        channels.push(ConfiguredChannel {
            display_name: "Linq",
            channel: Arc::new(LinqChannel::new(
                lq.api_token.clone(),
                lq.from_phone.clone(),
                lq.allowed_senders.clone(),
            )),
        });
    }

    #[cfg(feature = "channel-wati")]
    if let Some(ref wati_cfg) = config.channels_config.wati {
        let wati_channel = WatiChannel::new_with_proxy(
            wati_cfg.api_token.clone(),
            wati_cfg.api_url.clone(),
            wati_cfg.tenant_id.clone(),
            wati_cfg.allowed_numbers.clone(),
            wati_cfg.proxy_url.clone(),
        )
        .with_transcription(config.transcription.clone());

        channels.push(ConfiguredChannel {
            display_name: "WATI",
            channel: Arc::new(wati_channel),
        });
    }

    #[cfg(feature = "channel-nextcloud")]
    if let Some(ref nc) = config.channels_config.nextcloud_talk {
        channels.push(ConfiguredChannel {
            display_name: "Nextcloud Talk",
            channel: Arc::new(NextcloudTalkChannel::new_with_proxy(
                nc.base_url.clone(),
                nc.app_token.clone(),
                nc.bot_name.clone().unwrap_or_default(),
                nc.allowed_users.clone(),
                nc.proxy_url.clone(),
            )),
        });
    }

    #[cfg(feature = "channel-email")]
    if let Some(ref email_cfg) = config.channels_config.email {
        channels.push(ConfiguredChannel {
            display_name: "Email",
            channel: Arc::new(EmailChannel::new(email_cfg.clone())),
        });
    }

    #[cfg(not(feature = "channel-email"))]
    if config.channels_config.email.is_some() {
        tracing::warn!(
            "Email channel is configured but this build was compiled without `channel-email`; skipping."
        );
    }

    #[cfg(feature = "channel-email")]
    if let Some(ref gp_cfg) = config.channels_config.gmail_push {
        if gp_cfg.enabled {
            channels.push(ConfiguredChannel {
                display_name: "Gmail Push",
                channel: Arc::new(GmailPushChannel::new(gp_cfg.clone())),
            });
        }
    }

    #[cfg(not(feature = "channel-email"))]
    if config.channels_config.gmail_push.as_ref().is_some_and(|gp| gp.enabled) {
        tracing::warn!(
            "Gmail Push channel is configured but this build was compiled without `channel-email`; skipping."
        );
    }

    #[cfg(feature = "channel-irc")]
    if let Some(ref irc) = config.channels_config.irc {
        channels.push(ConfiguredChannel {
            display_name: "IRC",
            channel: Arc::new(IrcChannel::new(irc::IrcChannelConfig {
                server: irc.server.clone(),
                port: irc.port,
                nickname: irc.nickname.clone(),
                username: irc.username.clone(),
                channels: irc.channels.clone(),
                allowed_users: irc.allowed_users.clone(),
                server_password: irc.server_password.clone(),
                nickserv_password: irc.nickserv_password.clone(),
                sasl_password: irc.sasl_password.clone(),
                verify_tls: irc.verify_tls.unwrap_or(true),
            })),
        });
    }

    #[cfg(feature = "channel-lark")]
    if let Some(ref lk) = config.channels_config.lark {
        if lk.use_feishu {
            if config.channels_config.feishu.is_some() {
                tracing::warn!(
                    "Both [channels_config.feishu] and legacy [channels_config.lark].use_feishu=true are configured; ignoring legacy Feishu fallback in lark."
                );
            } else {
                tracing::warn!(
                    "Using legacy [channels_config.lark].use_feishu=true compatibility path; prefer [channels_config.feishu]."
                );
                channels.push(ConfiguredChannel {
                    display_name: "Feishu",
                    channel: Arc::new(
                        LarkChannel::from_config(lk)
                            .with_transcription(config.transcription.clone()),
                    ),
                });
            }
        } else {
            channels.push(ConfiguredChannel {
                display_name: "Lark",
                channel: Arc::new(
                    LarkChannel::from_lark_config(lk)
                        .with_transcription(config.transcription.clone()),
                ),
            });
        }
    }

    #[cfg(feature = "channel-lark")]
    if let Some(ref fs) = config.channels_config.feishu {
        channels.push(ConfiguredChannel {
            display_name: "Feishu",
            channel: Arc::new(
                LarkChannel::from_feishu_config(fs)
                    .with_transcription(config.transcription.clone()),
            ),
        });
    }

    #[cfg(not(feature = "channel-lark"))]
    if config.channels_config.lark.is_some() || config.channels_config.feishu.is_some() {
        tracing::warn!(
            "Lark/Feishu channel is configured but this build was compiled without `channel-lark`; skipping Lark/Feishu health check."
        );
    }

    #[cfg(feature = "channel-dingtalk")]
    if let Some(ref dt) = config.channels_config.dingtalk {
        channels.push(ConfiguredChannel {
            display_name: "DingTalk",
            channel: Arc::new(
                DingTalkChannel::new(
                    dt.client_id.clone(),
                    dt.client_secret.clone(),
                    dt.allowed_users.clone(),
                )
                .with_proxy_url(dt.proxy_url.clone()),
            ),
        });
    }

    #[cfg(feature = "channel-qq")]
    if let Some(ref qq) = config.channels_config.qq {
        channels.push(ConfiguredChannel {
            display_name: "QQ",
            channel: Arc::new(
                QQChannel::new(
                    qq.app_id.clone(),
                    qq.app_secret.clone(),
                    qq.allowed_users.clone(),
                )
                .with_workspace_dir(config.workspace_dir.clone())
                .with_proxy_url(qq.proxy_url.clone()),
            ),
        });
    }

    #[cfg(feature = "channel-twitter")]
    if let Some(ref tw) = config.channels_config.twitter {
        channels.push(ConfiguredChannel {
            display_name: "X/Twitter",
            channel: Arc::new(TwitterChannel::new(
                tw.bearer_token.clone(),
                tw.allowed_users.clone(),
            )),
        });
    }

    #[cfg(feature = "channel-mochat")]
    if let Some(ref mc) = config.channels_config.mochat {
        channels.push(ConfiguredChannel {
            display_name: "Mochat",
            channel: Arc::new(MochatChannel::new(
                mc.api_url.clone(),
                mc.api_token.clone(),
                mc.allowed_users.clone(),
                mc.poll_interval_secs,
            )),
        });
    }

    #[cfg(feature = "channel-wecom")]
    if let Some(ref wc) = config.channels_config.wecom {
        channels.push(ConfiguredChannel {
            display_name: "WeCom",
            channel: Arc::new(WeComChannel::new(
                wc.webhook_key.clone(),
                wc.allowed_users.clone(),
            )),
        });
    }

    #[cfg(feature = "channel-clawdtalk")]
    if let Some(ref ct) = config.channels_config.clawdtalk {
        channels.push(ConfiguredChannel {
            display_name: "ClawdTalk",
            channel: Arc::new(ClawdTalkChannel::new(ct.clone())),
        });
    }

    // Notion database poller channel
    #[cfg(feature = "channel-notion")]
    if config.notion.enabled && !config.notion.database_id.trim().is_empty() {
        let notion_api_key = if config.notion.api_key.trim().is_empty() {
            std::env::var("NOTION_API_KEY").unwrap_or_default()
        } else {
            config.notion.api_key.trim().to_string()
        };
        if notion_api_key.trim().is_empty() {
            tracing::warn!(
                "Notion channel enabled but no API key found (set notion.api_key or NOTION_API_KEY env var)"
            );
        } else {
            channels.push(ConfiguredChannel {
                display_name: "Notion",
                channel: Arc::new(NotionChannel::new(
                    notion_api_key,
                    config.notion.database_id.clone(),
                    config.notion.poll_interval_secs,
                    config.notion.status_property.clone(),
                    config.notion.input_property.clone(),
                    config.notion.result_property.clone(),
                    config.notion.max_concurrent,
                    config.notion.recover_stale,
                )),
            });
        }
    }

    #[cfg(feature = "channel-reddit")]
    if let Some(ref rd) = config.channels_config.reddit {
        channels.push(ConfiguredChannel {
            display_name: "Reddit",
            channel: Arc::new(RedditChannel::new(
                rd.client_id.clone(),
                rd.client_secret.clone(),
                rd.refresh_token.clone(),
                rd.username.clone(),
                rd.subreddit.clone(),
            )),
        });
    }

    #[cfg(feature = "channel-bluesky")]
    if let Some(ref bs) = config.channels_config.bluesky {
        channels.push(ConfiguredChannel {
            display_name: "Bluesky",
            channel: Arc::new(BlueskyChannel::new(
                bs.handle.clone(),
                bs.app_password.clone(),
            )),
        });
    }

    #[cfg(feature = "voice-wake")]
    if let Some(ref vw) = config.channels_config.voice_wake {
        channels.push(ConfiguredChannel {
            display_name: "VoiceWake",
            channel: Arc::new(VoiceWakeChannel::new(
                vw.clone(),
                config.transcription.clone(),
            )),
        });
    }

    #[cfg(feature = "channel-webhook")]
    if let Some(ref wh) = config.channels_config.webhook {
        channels.push(ConfiguredChannel {
            display_name: "Webhook",
            channel: Arc::new(WebhookChannel::new(
                wh.port,
                wh.listen_path.clone(),
                wh.send_url.clone(),
                wh.send_method.clone(),
                wh.auth_header.clone(),
                wh.secret.clone(),
            )),
        });
    }

    channels
}
